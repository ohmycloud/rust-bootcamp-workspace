use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use axum::response::IntoResponse;
use serde::{Deserialize, Serialize};
use crate::{AppError, AppState, User};
use crate::models::{CreateUser, SigninUser};

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthOutput {
    token: String,
}

#[axum::debug_handler]
pub(crate) async fn signup_handler(
    State(state): State<AppState>,
    Json(user): Json<CreateUser>
) -> Result<impl IntoResponse, AppError> {
    let user = User::create(&user, &state.pool).await?;
    let token = state.ek.sign(user)?;
    let body = Json(AuthOutput { token });

    Ok((StatusCode::CREATED, body))
}

pub(crate) async fn signin_handler(
    State(state): State<AppState>,
    Json(user): Json<SigninUser>
) -> Result<impl IntoResponse, AppError> {
    let user = User::verify(&user, &state.pool).await?;
    match user {
        Some(user) => {
            let token = state.ek.sign(user)?;
            Ok((StatusCode::CREATED, token).into_response())
        },
        None => Ok((StatusCode::FORBIDDEN, "Invalid email or password".to_string()).into_response()),
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use http_body_util::BodyExt;
    use anyhow::Result;
    use jwt_simple::reexports::serde_json;
    use crate::AppConfig;

    #[tokio::test]
    async fn signup_should_work() -> Result<()> {
        let config = AppConfig::load()?;
        let (tdb, state) = AppState::new_for_test(config).await?;
        let created_user = CreateUser::new("ohmycoudy", "ohmycloudy@uk", "hunter42");
        let ret = signup_handler(State(state), Json(created_user))
            .await?
            .into_response();

        assert_eq!(ret.status(), StatusCode::CREATED);
        let body = ret.into_body().collect().await?.to_bytes();
        let ret: serde_json::Value = serde_json::from_slice(&body)?;
        assert!(ret["token"].is_string());

        Ok(())
    }
}
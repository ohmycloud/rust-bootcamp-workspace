use crate::models::{CreateUser, SigninUser};
use crate::{AppError, AppState, User};
use axum::Json;
use axum::extract::State;
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::response::IntoResponse;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthOutput {
    token: String,
}

#[axum::debug_handler]
pub(crate) async fn signup_handler(
    State(state): State<AppState>,
    Json(user): Json<CreateUser>,
) -> Result<impl IntoResponse, AppError> {
    let user = User::create(&user, &state.pool).await?;
    let token = state.ek.sign(user)?;
    let mut header = HeaderMap::new();
    header.insert("X-Token", HeaderValue::from_str(&token)?);
    let body = Json(AuthOutput { token });

    Ok((StatusCode::CREATED, body))
}

pub(crate) async fn signin_handler(
    State(state): State<AppState>,
    Json(user): Json<SigninUser>,
) -> Result<impl IntoResponse, AppError> {
    let user = User::verify(&user, &state.pool).await?;
    match user {
        Some(user) => {
            let token = state.ek.sign(user)?;
            Ok((StatusCode::CREATED, token).into_response())
        }
        None => Ok((
            StatusCode::FORBIDDEN,
            "Invalid email or password".to_string(),
        )
            .into_response()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AppConfig;
    use anyhow::{Context, Result};
    use http_body_util::BodyExt;

    #[tokio::test]
    async fn signup_should_work() -> Result<()> {
        let config = AppConfig::load()?;
        let (_tdb, state) = AppState::new_for_test(config).await?;
        let created_user = CreateUser::new("raku", "raku@dev.org", "Hunter42");

        let ret = signup_handler(State(state), Json(created_user))
            .await?
            .into_response();

        assert_eq!(ret.status(), StatusCode::CREATED);
        let body = ret.into_body().collect().await?.to_bytes();
        let ret: AuthOutput = serde_json::from_slice(&body)?;
        assert_ne!(ret.token, "");

        let token = String::from_utf8(body.to_vec())?;
        assert_eq!(token, "");

        Ok(())
    }

    #[tokio::test]
    async fn signin_should_work() -> Result<()> {
        let config = AppConfig::load()?;
        let (_tdb, state) = AppState::new_for_test(config).await?;
        let name = "Alice";
        let email = "alice@acme.org";
        let password = "Hunter42";

        let user = CreateUser::new(name, email, password);
        User::create(&user, &state.pool).await?;
        let input = SigninUser::new(email, password);
        let ret = signin_handler(State(state), Json(input))
            .await?
            .into_response();
        assert_eq!(ret.status(), StatusCode::CREATED);
        let body = ret.into_body().collect().await?.to_bytes();
        let ret: AuthOutput = serde_json::from_slice(&body)?;
        assert_ne!(ret.token, "");
        Ok(())
    }
}

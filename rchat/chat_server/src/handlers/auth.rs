use crate::error::ErrorOutput;
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
            Ok((StatusCode::CREATED, Json(AuthOutput { token })).into_response())
        }
        None => {
            let body = Json(ErrorOutput::new("Invalid email or password"));
            Ok((StatusCode::FORBIDDEN, body).into_response())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::ErrorOutput;
    use anyhow::Result;
    use http_body_util::BodyExt;

    #[tokio::test]
    async fn signup_should_work() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let created_user = CreateUser::new("raku", "none", "raku@dev.org", "Hunter42");

        let ret = signup_handler(State(state), Json(created_user))
            .await?
            .into_response();

        assert_eq!(ret.status(), StatusCode::CREATED);
        let body = ret.into_body().collect().await?.to_bytes();
        let ret: AuthOutput = serde_json::from_slice(&body)?;
        assert_ne!(ret.token, "");

        Ok(())
    }

    #[tokio::test]
    async fn signing_should_work() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let name = "Alice";
        let email = "alice1@acme.org";
        let password = "Hunter42";

        let user = CreateUser::new(name, "none", email, password);
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

    #[tokio::test]
    async fn signing_with_non_exist_user_should_403() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let email = "wangwei@tang.org";
        let password = "123456";
        let signing_user = SigninUser::new(email, password);
        let ret = signin_handler(State(state), Json(signing_user))
            .await
            .into_response();
        assert_eq!(ret.status(), StatusCode::FORBIDDEN);

        let body = ret.into_body().collect().await?.to_bytes();
        let ret: ErrorOutput = serde_json::from_slice(&body)?;
        assert_eq!(ret.error, "Invalid email or password");

        Ok(())
    }

    #[tokio::test]
    async fn signup_duplicate_user_should_409() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let input = CreateUser::new("Alice Chen", "dev", "alice@acme.org", "123456");

        let ret = signup_handler(State(state), Json(input))
            .await
            .into_response();
        assert_eq!(ret.status(), StatusCode::CONFLICT);
        let body = ret.into_body().collect().await?.to_bytes();
        let ret: ErrorOutput = serde_json::from_slice(&body)?;

        assert_eq!(ret.error, "email already exists: alice@acme.org");
        Ok(())
    }
}

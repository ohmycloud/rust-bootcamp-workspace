use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use axum::response::IntoResponse;
use serde::{Deserialize, Serialize};
use crate::{AppError, AppState, User};
use crate::models::CreateUser;

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthOutput {
    token: String,
}

#[axum::debug_handler]
pub(crate) async fn signin_handler(
    State(state): State<AppState>,
    Json(user): Json<CreateUser>
) -> Result<impl IntoResponse, AppError> {
    let user = User::create(&user, &state.pool).await?;
    let token = state.ek.sign(user)?;
    let body = Json(AuthOutput { token });

    Ok((StatusCode::CREATED, body))
}

pub(crate) async fn signup_handler() -> impl IntoResponse {
    "signup"
}

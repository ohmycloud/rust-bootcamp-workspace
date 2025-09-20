use crate::models::ChatUser;
use crate::{AppError, AppState, User};
use axum::extract::State;
use axum::response::IntoResponse;
use axum::{Extension, Json};

/// List all users in the workspace.
#[utoipa::path(
    get,
    path = "/api/users",
    responses(
        (status = 200, description = "List of ws users", body = Vec<ChatUser>)
    ),
    security(
        ("token" = [])
    )
)]
pub(crate) async fn list_chat_users_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let users = state.fetch_all_chat_users(user.ws_id).await?;
    Ok(Json(users))
}

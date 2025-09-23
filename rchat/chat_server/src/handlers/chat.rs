use crate::AppState;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Extension, Json};
use chat_core::{AppError, Chat, CreateChat, ErrorOutput, User};

/// List all chats in the workspace of the user.
#[utoipa::path(
    get,
    path = "/api/chats",
    responses((status = 200, description = "List of chats", body = Vec<Chat>)),
    security(("token" = []))
)]
pub(crate) async fn list_chat_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let chats = state.fetch_all_chats(user.ws_id).await?;
    Ok((StatusCode::OK, Json(chats)))
}

/// Create a new chat in the workspace of the user.
#[utoipa::path(
    post,
    path = "/api/chats",
    responses((status = 201, description = "Chat created successfully", body = Chat)),
    security(("token" = []))
)]
pub(crate) async fn create_chat_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
    Json(chat): Json<CreateChat>,
) -> Result<impl IntoResponse, AppError> {
    let chat = state.create_chat(chat, user.ws_id).await?;
    Ok((StatusCode::CREATED, Json(chat)))
}

/// Get a chat info by id.
#[utoipa::path(
    get,
    path = "/api/chats/{id}",
    params(("id" = u64, description = "Chat id")),
    responses(
        (status = 200, description = "Chat retrieved successfully", body = Chat),
        (status = 404, description = "Chat not found", body = ErrorOutput),
    ),
    security(("token" = []))
)]
pub(crate) async fn get_chat_handler(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    let chat = state.fetch_chat_by_id(id).await?;
    match chat {
        Some(chat) => Ok(Json(chat)),
        None => Err(AppError::UserNotFound(format!("chat id {} not found", id))),
    }
}

pub(crate) async fn delete_chat_handler() -> impl IntoResponse {
    "delete chat"
}

pub(crate) async fn update_chat_handler() -> impl IntoResponse {
    "update chat"
}

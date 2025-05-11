use crate::models::{Chat, CreateChat};
use crate::{AppError, AppState, User};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Extension, Json};

#[allow(unused)]
pub(crate) async fn list_chat_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let chat = Chat::fetch_all(user.ws_id, &state.pool).await?;
    Ok((StatusCode::OK, Json(chat)))
}

pub(crate) async fn create_chat_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
    Json(chat): Json<CreateChat>,
) -> Result<impl IntoResponse, AppError> {
    let chat = Chat::create(chat, user.ws_id, &state.pool).await?;
    Ok((StatusCode::CREATED, Json(chat)))
}

pub(crate) async fn get_chat_handler(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    let chat = Chat::fetch_by_id(id, &state.pool).await?;
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

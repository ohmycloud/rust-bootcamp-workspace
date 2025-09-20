use crate::error::ErrorOutput;
use crate::models::{ChatFile, ListMessages, Message};
use crate::{AppError, AppState, User};
use axum::extract::{Multipart, Path, State};
use axum::http::HeaderMap;
use axum::response::IntoResponse;
use axum::{Extension, Json};
use tokio::fs;
use tracing::{info, warn};

/// Send a new message to the chat.
#[utoipa::path(
    post,
    path ="/api/chats/{id}",
    params(
        ("id" = u64, Path, description = "Chat ID")
    ),
    responses(
        (status = 200, description = "List of Messages", body = Message),
        (status = 400, description = "Invalid input", body = ErrorOutput)
    ),
    security(
        ("token" = [])
    )
)]
pub(crate) async fn send_message_handler() -> impl IntoResponse {
    "send message"
}

/// List all messages in the chat.
#[utoipa::path(
    get,
    path ="/api/chats/{id}/messages",
    params(
        ("id" = u64, Path, description = "Chat ID"),
        ListMessages
    ),
    responses(
        (status = 200, description = "List of messages", body = Vec<Message>),
        (status = 400, description = "Invalid input", body = ErrorOutput)
    ),
    security(
        ("token" = [])
    )
)]
pub(crate) async fn list_message_handler() -> impl IntoResponse {
    "list message"
}

pub(crate) async fn upload_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, AppError> {
    let ws_id = user.ws_id;
    let base_dir = &state.config.server.base_dir;
    let mut files = vec![];

    while let Some(field) = multipart.next_field().await.unwrap() {
        let filename = field.file_name().map(|name| name.to_string());
        let (Some(filename), Ok(data)) = (filename, field.bytes().await) else {
            warn!("Failed to read multipart",);
            continue;
        };

        let file = ChatFile::new(ws_id, &filename, &data);
        let path = file.path(&base_dir);
        if path.exists() {
            info!("File {} already exists: {:?}", filename, path);
            continue;
        } else {
            fs::create_dir_all(path.parent().expect("file path parent should exists")).await?;
            fs::write(path, data).await?;
        }
        files.push(file.url());
    }

    Ok(Json(files))
}

pub(crate) async fn download_file_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
    Path((ws_id, path)): Path<(i64, String)>,
) -> Result<impl IntoResponse, AppError> {
    if user.ws_id != ws_id {
        return Err(AppError::UserNotFound("File doesn't exist".to_string()));
    }
    let base_dir = state.config.server.base_dir.join(ws_id.to_string());
    let path = base_dir.join(path);
    if !path.exists() {
        return Err(AppError::UserNotFound(format!(
            "File {} not found",
            path.to_string_lossy()
        )));
    }

    let mime = mime_guess::from_path(&path).first_or_octet_stream();
    let body = fs::read(path).await?;
    let mut headers = HeaderMap::new();
    headers.insert("content-type", mime.to_string().parse().unwrap());

    Ok((headers, body))
}

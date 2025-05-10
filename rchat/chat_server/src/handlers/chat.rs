use axum::Extension;
use axum::response::IntoResponse;
use crate::User;

#[allow(unused)]
pub(crate) async fn list_chat_handler(
    Extension(user): Extension<User>
) -> impl IntoResponse {
    user.fullname
}

pub(crate) async fn create_chat_handler() -> impl IntoResponse {
    "create chat"
}

pub(crate) async fn delete_chat_handler() -> impl IntoResponse {
    "delete chat"
}

pub(crate) async fn update_chat_handler() -> impl IntoResponse {
    "update chat"
}

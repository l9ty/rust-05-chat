use axum::{response::IntoResponse, Extension};

pub async fn list_chat_handler(Extension(uid): Extension<i64>) -> impl IntoResponse {
    format!("list chat {}", uid)
}

pub async fn create_chat_handler() -> impl IntoResponse {
    "create chat"
}

pub async fn update_chat_handler() -> impl IntoResponse {
    "update chat"
}

pub async fn delete_chat_handler() -> impl IntoResponse {
    "delete chat"
}

pub async fn send_message_handler() -> impl IntoResponse {
    "send message"
}

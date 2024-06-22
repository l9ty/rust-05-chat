use axum::response::IntoResponse;

pub async fn list_chat_handler() -> impl IntoResponse {
    "list chat"
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

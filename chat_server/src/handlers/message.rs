use axum::response::IntoResponse;

pub async fn list_message_handler() -> impl IntoResponse {
    "list message"
}

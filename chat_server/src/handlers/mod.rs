use axum::response::IntoResponse;

mod auth;
mod chat;
mod message;

pub use auth::*;
pub use chat::*;
pub use message::*;

pub async fn index_handler() -> impl IntoResponse {
    "index"
}

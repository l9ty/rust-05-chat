use axum::response::IntoResponse;

mod auth;
pub use auth::{signin_handler, singup_handler};

pub(crate) async fn index_handler() -> impl IntoResponse {
    "index"
}

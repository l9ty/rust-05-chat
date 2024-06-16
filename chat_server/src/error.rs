use axum::{
    body::Body,
    http::{Response, StatusCode},
    response::IntoResponse,
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("db error: {0}")]
    SqlxError(#[from] sqlx::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response<Body> {
        StatusCode::INTERNAL_SERVER_ERROR.into_response()
    }
}

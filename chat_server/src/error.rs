use axum::{
    response::{IntoResponse, Response},
    Json,
};
use http::StatusCode;
use serde_json::json;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug)]
pub enum AppError {
    InvalidInput(String),
    EntityExist(String),
    Any(anyhow::Error),
}

impl<T> From<T> for AppError
where
    T: Into<anyhow::Error>,
{
    fn from(err: T) -> Self {
        AppError::Any(err.into())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (code, err) = match self {
            AppError::InvalidInput(err) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                format!("invalid input: {err}"),
            ),
            AppError::EntityExist(err) => (StatusCode::CONFLICT, format!("{err} exists")),
            AppError::Any(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
        };

        (code, Json(json!({ "error": err }))).into_response()
    }
}

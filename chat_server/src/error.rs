use axum::{
    body::Body,
    http::{Response, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("db error: {0}")]
    SqlxError(#[from] sqlx::Error),

    #[error("jwt error: {0}")]
    JwtError(#[from] jsonwebtoken::errors::Error),
    // todo #[from] argon2 error cause compile error
    #[error("password hash error: {0}")]
    PasswordHashError(String),

    #[error("user already exist")]
    UserAlreadyExist,
}

#[derive(Serialize, Deserialize)]
pub struct RespError {
    error: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response<Body> {
        let status = match self {
            Self::UserAlreadyExist => StatusCode::CONFLICT,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status, Json(RespError::new(self.to_string()))).into_response()
    }
}

impl RespError {
    pub fn new(error: impl AsRef<str>) -> Self {
        Self {
            error: error.as_ref().to_string(),
        }
    }
}

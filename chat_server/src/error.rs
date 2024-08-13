use std::fmt;

use axum::{
    response::{IntoResponse, Response},
    Json,
};
use http::StatusCode;
use serde::Serialize;
use serde_json::json;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, Clone, Serialize)]
pub enum ImmutStr {
    Static(&'static str),
    Boxed(Box<str>),
}

#[derive(Debug)]
pub enum AppError {
    NotFound(ImmutStr),
    InvalidInput(ImmutStr),
    AlreadyExist(ImmutStr),
    Forbidden(ImmutStr),
    Internal(anyhow::Error),
}

impl<T> From<T> for AppError
where
    T: Into<anyhow::Error>,
{
    fn from(err: T) -> Self {
        AppError::Internal(err.into())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (code, err) = match self {
            Self::NotFound(err) => (StatusCode::NOT_FOUND, err),
            Self::InvalidInput(err) => (StatusCode::UNPROCESSABLE_ENTITY, err),
            Self::AlreadyExist(err) => (StatusCode::CONFLICT, err),
            Self::Forbidden(err) => (StatusCode::FORBIDDEN, err),
            Self::Internal(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string().into()),
        };

        (code, Json(json!({ "error": err }))).into_response()
    }
}

impl AppError {
    #[inline]
    pub fn not_found(msg: impl Into<ImmutStr>) -> Self {
        AppError::NotFound(msg.into())
    }

    #[inline]
    pub fn invalid_input(msg: impl Into<ImmutStr>) -> Self {
        AppError::InvalidInput(msg.into())
    }

    #[inline]
    pub fn already_exist(msg: impl Into<ImmutStr>) -> Self {
        AppError::AlreadyExist(msg.into())
    }

    #[inline]
    pub fn forbidden(msg: impl Into<ImmutStr>) -> Self {
        AppError::Forbidden(msg.into())
    }

    #[inline]
    pub fn any(e: impl Into<anyhow::Error>) -> Self {
        AppError::Internal(e.into())
    }
}

impl From<&'static str> for ImmutStr {
    #[inline]
    fn from(s: &'static str) -> Self {
        ImmutStr::Static(s)
    }
}

impl From<String> for ImmutStr {
    #[inline]
    fn from(s: String) -> Self {
        ImmutStr::Boxed(s.into_boxed_str())
    }
}

impl fmt::Display for ImmutStr {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ImmutStr::Static(s) => s.fmt(f),
            ImmutStr::Boxed(s) => s.fmt(f),
        }
    }
}

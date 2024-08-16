use std::fmt;

use serde::Serialize;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, Clone, Serialize)]
pub enum ImmutStr {
    Static(&'static str),
    Boxed(Box<str>),
}

#[derive(Debug)]
pub enum AppError {
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

// impl IntoResponse for AppError {
//     fn into_response(self) -> Response {
//         let (code, err) = match self {
//             Self::Internal(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string().into()),
//         };
//
//         (code, Json(json!({ "error": err }))).into_response()
//     }
// }

impl AppError {
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

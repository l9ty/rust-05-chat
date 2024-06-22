use axum::{
    body::Body,
    extract::{FromRequestParts, Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use axum_extra::TypedHeader;
use headers::{authorization::Bearer, Authorization};
use tracing::warn;

use crate::AppState;

// TODO: rename token header

pub async fn verify_token(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Response<Body> {
    let (mut parts, body) = req.into_parts();
    let header = TypedHeader::<Authorization<Bearer>>::from_request_parts(&mut parts, &state).await;
    let token = match header {
        Ok(TypedHeader(Authorization(token))) => token,
        Err(e) => {
            warn!("failed to parse bearer header: {:?}", e);
            return (StatusCode::UNAUTHORIZED, "unauthorized").into_response();
        }
    };

    let user = match state.dk.verify(token.token()) {
        Ok(user) => user,
        Err(e) => {
            warn!("invalid bearer token {e}");
            return (StatusCode::FORBIDDEN, "invalid token").into_response();
        }
    };

    parts.extensions.insert(user);
    next.run(Request::from_parts(parts, body)).await
}

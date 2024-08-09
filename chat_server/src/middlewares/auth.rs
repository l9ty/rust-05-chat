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

#[cfg(test)]
mod test {
    use axum::{middleware::from_fn_with_state, routing::get, Router};
    use http::Request;
    use tower::ServiceExt;

    use super::*;

    #[axum::debug_handler]
    async fn index_handler() -> &'static str {
        "hello"
    }

    #[sqlx::test]
    async fn verify_token_should_work(pool: sqlx::PgPool) -> anyhow::Result<()> {
        let state = AppState::new_for_test(pool);
        let router = Router::new()
            .route("/", get(index_handler))
            .layer(from_fn_with_state(state.clone(), verify_token));

        let token = state.ek.sign(&Default::default())?;

        // good token
        let req = Request::builder()
            .uri("/")
            .header("Authorization", format!("Bearer {token}"))
            .body(Body::empty());

        let resp = router.clone().oneshot(req?).await?;
        assert_eq!(resp.status(), StatusCode::OK);

        // bad token
        let req = Request::builder()
            .uri("/")
            .header("Authorization", "Bearer nothing")
            .body(Body::empty());

        let resp = router.clone().oneshot(req?).await?;
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);

        // missing token
        let req = Request::builder().uri("/").body(Body::empty());
        let resp = router.clone().oneshot(req?).await?;
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

        Ok(())
    }
}

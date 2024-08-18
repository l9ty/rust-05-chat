use std::borrow::Cow;

use axum::{
    body::Body,
    extract::{FromRequestParts, Query, Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use axum_extra::TypedHeader;
use headers::{authorization::Bearer, Authorization};
use serde::Deserialize;

use super::VerifyToken;

#[derive(Deserialize, Default)]
struct QueryParams {
    access_token: String,
}

pub async fn verify_token<S>(State(state): State<S>, req: Request, next: Next) -> Response<Body>
where
    S: VerifyToken + Clone + Send + Sync + 'static,
{
    let (mut parts, body) = req.into_parts();
    let header = TypedHeader::<Authorization<Bearer>>::from_request_parts(&mut parts, &state).await;

    let token = match header {
        Ok(TypedHeader(Authorization(ref token))) => Cow::Borrowed(token.token()),
        Err(e) => {
            if !e.is_missing() {
                return (StatusCode::UNAUTHORIZED, "cannot parse token from header")
                    .into_response();
            }
            let query = Query::<QueryParams>::from_request_parts(&mut parts, &state)
                .await
                .unwrap_or_default();
            Cow::Owned(query.0.access_token)
        }
    };

    if token.is_empty() {
        return (StatusCode::UNAUTHORIZED, "missing token").into_response();
    }

    let user = match state.verify(token.as_ref()) {
        Ok(user) => user,
        Err(_) => {
            return (StatusCode::FORBIDDEN, "invalid token").into_response();
        }
    };

    parts.extensions.insert(user);
    next.run(Request::from_parts(parts, body)).await
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use axum::{middleware::from_fn_with_state, routing::get, Router};
    use http::Request;
    use tower::ServiceExt;

    use crate::utils::{JwtDecodingKey, JwtEncodingKey};

    use super::*;

    #[derive(Clone)]
    struct AppState {
        inner: Arc<AppStateInner>,
    }

    struct AppStateInner {
        ek: JwtEncodingKey,
        dk: JwtDecodingKey,
    }

    impl VerifyToken for AppState {
        type Error = anyhow::Error;

        fn verify(&self, token: &str) -> Result<crate::utils::UserCliams, Self::Error> {
            self.inner.dk.verify(token)
        }
    }

    async fn index_handler() -> &'static str {
        "hello"
    }

    const EK: &[u8] = include_bytes!("../../../fixtures/private.pem");
    const DK: &[u8] = include_bytes!("../../../fixtures/public.pem");

    #[tokio::test]
    async fn t_verify_token() {
        println!("{}", String::from_utf8_lossy(EK));

        let state = AppState {
            inner: Arc::new(AppStateInner {
                ek: JwtEncodingKey::load(EK).unwrap(),
                dk: JwtDecodingKey::load(DK).unwrap(),
            }),
        };

        let router = Router::new()
            .route("/", get(index_handler))
            .layer(from_fn_with_state(state.clone(), verify_token::<AppState>));

        // good token
        let token = state.inner.ek.sign(&Default::default()).unwrap();
        let req = Request::builder()
            .uri("/")
            .header("Authorization", format!("Bearer {token}"))
            .body(Body::empty());
        let resp = router.clone().oneshot(req.unwrap()).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        // good token in query
        let req = Request::builder()
            .uri(format!("/?access_token={token}"))
            .body(Body::empty());
        let resp = router.clone().oneshot(req.unwrap()).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        // bad token
        let req = Request::builder()
            .uri("/")
            .header("Authorization", "Bearer nothing")
            .body(Body::empty());
        let resp = router.clone().oneshot(req.unwrap()).await.unwrap();
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);

        // bad token in query
        let req = Request::builder()
            .uri("/?access_token=nothing")
            .body(Body::empty());
        let resp = router.clone().oneshot(req.unwrap()).await.unwrap();
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);

        // missing token
        let req = Request::builder().uri("/").body(Body::empty());
        let resp = router.clone().oneshot(req.unwrap()).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }
}

use axum::{extract::Request, http::HeaderValue, response::Response};
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use tower::{Layer, Service};
use tracing::warn;

use super::SERVER_TIME_HEADER;

#[derive(Clone)]
pub struct ServerTimeLayer;

impl<S> Layer<S> for ServerTimeLayer {
    type Service = ServerTimeMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        ServerTimeMiddleware { inner }
    }
}

#[derive(Clone)]
pub struct ServerTimeMiddleware<S> {
    inner: S,
}

impl<S> Service<Request> for ServerTimeMiddleware<S>
where
    S: Service<Request, Response = Response> + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    // `BoxFuture` is a type alias for `Pin<Box<dyn Future + Send + 'a>>`
    type Future =
        Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>>;
    // type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request) -> Self::Future {
        let start = tokio::time::Instant::now();
        let future = self.inner.call(req);
        Box::pin(async move {
            let mut resp: Response = future.await?;

            let duration = start.elapsed();
            let elapsed = duration.as_millis();
            let server_time_str = format!("{elapsed}ms");
            let server_time = HeaderValue::from_str(&server_time_str);

            match server_time {
                Ok(server_time) => {
                    resp.headers_mut().insert(SERVER_TIME_HEADER, server_time);
                }
                Err(err) => {
                    warn!("parse server time error: {server_time_str}, error: {err}");
                }
            }

            Ok(resp)
        })
    }
}

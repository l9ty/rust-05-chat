mod auth;
mod request_id;
mod server_time;

use axum::{middleware::from_fn, Router};
use server_time::ServerTimeLayer;
use tower_http::{
    compression::CompressionLayer,
    trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer},
    LatencyUnit,
};

pub use auth::verify_token;
use request_id::set_request_id;
use tower::ServiceBuilder;
use tracing::Level;

pub const REQUEST_ID_HEADER: &str = "X-Request-Id";
pub const SERVER_TIME_HEADER: &str = "X-Server-Time";

pub fn set_global_layer(app: Router) -> Router {
    let service = ServiceBuilder::new()
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().include_headers(true))
                .on_request(DefaultOnRequest::new().level(Level::INFO))
                .on_response(
                    DefaultOnResponse::new()
                        .level(Level::INFO)
                        .latency_unit(LatencyUnit::Micros),
                ),
        )
        .layer(CompressionLayer::new().gzip(true).br(true).deflate(true))
        .layer(from_fn(set_request_id))
        .layer(ServerTimeLayer);
    app.layer(service)
}

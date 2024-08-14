mod auth;
mod request_id;
mod server_time;

use axum::Router;
use http::HeaderName;
use request_id::RequestIdMaker;
use server_time::ServerTimeLayer;
use tower_http::{
    compression::CompressionLayer,
    request_id::{PropagateRequestIdLayer, SetRequestIdLayer},
    trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer},
    LatencyUnit,
};

pub use auth::verify_token;

// use request_id::set_request_id;
use tower::ServiceBuilder;
use tracing::Level;

use crate::utils::UserCliams;

pub const REQUEST_ID_HEADER: &str = "x-request-id";
pub const SERVER_TIME_HEADER: &str = "x-server-time";

/// Set global request_id, server_time, CompressionLayer, TraceLayer
pub fn set_global_layer(app: Router) -> Router {
    // NOTE header chars h2 require lowercase header
    let x_request_id = HeaderName::from_static(REQUEST_ID_HEADER);

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
        .layer(SetRequestIdLayer::new(x_request_id.clone(), RequestIdMaker))
        .layer(PropagateRequestIdLayer::new(x_request_id))
        .layer(ServerTimeLayer);
    app.layer(service)
}

pub trait VerifyToken {
    type Error: std::fmt::Debug;
    fn verify(&self, token: &str) -> Result<UserCliams, Self::Error>;
}

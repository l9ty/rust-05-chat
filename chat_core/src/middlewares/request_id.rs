use axum::{extract::Request, http::HeaderValue};

use tower_http::request_id::{MakeRequestId, RequestId};
use uuid::Uuid;

// Layer must implement Clone
#[derive(Clone)]
pub struct RequestIdMaker;

impl MakeRequestId for RequestIdMaker {
    fn make_request_id<B>(&mut self, _request: &Request<B>) -> Option<RequestId> {
        let uid = Uuid::now_v7().to_string();
        HeaderValue::from_str(&uid).ok().map(RequestId::new)
    }
}

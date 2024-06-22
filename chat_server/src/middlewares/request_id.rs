use axum::{extract::Request, http::HeaderValue, middleware::Next, response::Response};

use tracing::warn;
use uuid::Uuid;

use super::REQUEST_ID_HEADER;

pub async fn set_request_id(mut request: Request, next: Next) -> Response {
    let rid = match request.headers().get(REQUEST_ID_HEADER) {
        Some(rid) => Some(rid.clone()),
        None => {
            let uid = Uuid::now_v7().to_string();
            match HeaderValue::from_str(&uid) {
                Ok(rid) => {
                    request.headers_mut().insert(REQUEST_ID_HEADER, rid.clone());
                    Some(rid)
                }
                Err(e) => {
                    warn!("cannot generate request id: {}", e);
                    None
                }
            }
        }
    };

    let mut resp = next.run(request).await;

    let Some(rid) = rid else {
        return resp;
    };

    resp.headers_mut().insert(REQUEST_ID_HEADER, rid);
    resp
}

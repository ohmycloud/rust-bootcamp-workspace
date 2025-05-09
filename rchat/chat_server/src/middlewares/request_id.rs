use axum::extract::Request;
use axum::http::HeaderValue;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use tracing::warn;

const REQUEST_ID_HEADER : &str = "x-request-id";

pub async fn set_request_id(mut req: Request, next: Next) -> Response {
    let request_id = match req.headers().get(REQUEST_ID_HEADER) {
        Some(hv) => hv.as_bytes().to_vec(),
        None => {
            let request_id = uuid::Uuid::now_v7().to_string();
            match request_id.parse() {
                Ok(v) => {
                    req.headers_mut().insert(
                        REQUEST_ID_HEADER,
                        v
                    );
                }
                Err(e) => {
                    warn!("parse generated request id failed: {}", e);
                }
            }
            request_id.as_bytes().to_vec()
        }
    };

    let mut res = next.run(req).await;
    match HeaderValue::from_bytes(&request_id) {
        Ok(v) => {
            res.headers_mut().insert(
                REQUEST_ID_HEADER,
                v
            );
        }
        Err(e) => {
            warn!("parse generated request id failed: {}", e);
        }
    }

    res
}
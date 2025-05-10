use axum::extract::Request;
use axum::http::HeaderValue;
use axum::middleware::Next;
use axum::response::Response;
use tracing::warn;
use crate::middlewares::REQUEST_ID_HEADER;

pub async fn set_request_id(mut req: Request, next: Next) -> Response {
    let request_id = match req.headers().get(REQUEST_ID_HEADER) {
        Some(hv) => Some(hv.clone()),
        None => {
            let req_id = uuid::Uuid::now_v7().to_string();
            match req_id.parse::<HeaderValue>() {
                Ok(hv) => {
                    req.headers_mut().insert(REQUEST_ID_HEADER, hv.clone());
                    Some(hv)
                }
                Err(e) => {
                    warn!("parse generated request id failed: {}", e);
                    None
                }
            }
        }
    };

    let mut res = next.run(req).await;
    let Some(request_id) = request_id else {
        return res;
    };
    res.headers_mut().insert(REQUEST_ID_HEADER, request_id);
    res
}
use axum::http::Request;
use tower_http::request_id::{MakeRequestId, RequestId};
use uuid::Uuid;

#[derive(Clone, Copy)]
pub struct MakeRequestUuidV7;
impl MakeRequestId for MakeRequestUuidV7 {
    fn make_request_id<B>(&mut self, _request: &Request<B>) -> Option<RequestId> {
        // Use UUIDv7 so that request ID can be sorted by time
        let request_id = Uuid::now_v7();
        Some(RequestId::new(request_id.to_string().parse().unwrap()))
    }
}

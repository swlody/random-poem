use axum::{
    http::{HeaderName, Request},
    Router,
};
use tower_http::{
    request_id::{MakeRequestId, PropagateRequestIdLayer, RequestId, SetRequestIdLayer},
    trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer},
};
use uuid::Uuid;

#[allow(clippy::module_name_repetitions)]
pub trait AddLayers {
    fn add_tracing_layer(self) -> Router;
}

#[derive(Clone, Copy)]
pub struct MakeRequestUuidV7;

impl MakeRequestId for MakeRequestUuidV7 {
    fn make_request_id<B>(&mut self, _request: &Request<B>) -> Option<RequestId> {
        let request_id = Uuid::now_v7().to_string().parse().unwrap();
        Some(RequestId::new(request_id))
    }
}

impl AddLayers for Router {
    fn add_tracing_layer(self) -> Self {
        self.layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().include_headers(true))
                .on_response(DefaultOnResponse::new().include_headers(true)),
        )
        .layer(SetRequestIdLayer::new(
            HeaderName::from_static("x-request-id"),
            MakeRequestUuidV7,
        ))
        .layer(PropagateRequestIdLayer::x_request_id())
    }
}

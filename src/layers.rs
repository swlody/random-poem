use axum::{
    http::{HeaderName, Request},
    Router,
};
use sentry_tower::{NewSentryLayer, SentryHttpLayer};
use tower::ServiceBuilder;
use tower_http::{
    request_id::{MakeRequestId, PropagateRequestIdLayer, RequestId, SetRequestIdLayer},
    trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer},
};
use uuid::Uuid;

#[allow(clippy::module_name_repetitions)]
pub trait AddLayers {
    fn with_tracing_layer(self) -> Router;
    fn with_sentry_layer(self) -> Router;
}

#[derive(Clone, Copy)]
pub struct MakeRequestUuidV7;
impl MakeRequestId for MakeRequestUuidV7 {
    fn make_request_id<B>(&mut self, _request: &Request<B>) -> Option<RequestId> {
        // Use UUIDv7 so that request ID can be sorted by time
        let request_id = Uuid::now_v7().to_string().parse().unwrap();
        Some(RequestId::new(request_id))
    }
}

impl AddLayers for Router {
    fn with_tracing_layer(self) -> Self {
        // Enables tracing for each request and adds a request ID header to resposne
        let tracing_service = ServiceBuilder::new()
            .layer(
                TraceLayer::new_for_http()
                    .make_span_with(DefaultMakeSpan::new().include_headers(true))
                    .on_response(DefaultOnResponse::new().include_headers(true)),
            )
            .layer(SetRequestIdLayer::new(
                HeaderName::from_static("x-request-id"),
                MakeRequestUuidV7,
            ))
            .layer(PropagateRequestIdLayer::x_request_id());
        self.layer(tracing_service)
    }

    fn with_sentry_layer(self) -> Self {
        let sentry_service = ServiceBuilder::new()
            .layer(NewSentryLayer::new_from_top())
            .layer(SentryHttpLayer::with_transaction());
        self.layer(sentry_service)
    }
}

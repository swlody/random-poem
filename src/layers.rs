use axum::{http::Request, Router};
use sentry::integrations::tower::{NewSentryLayer, SentryHttpLayer};
use tower::ServiceBuilder;
use tower_http::{
    request_id::{MakeRequestId, RequestId},
    trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer},
    ServiceBuilderExt as _,
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
        let request_id = Uuid::now_v7();
        // TODO This is more appropriately addded along with other sentry layers.
        // Create a new middleware to read x-request-id and add it to sentry scope.
        sentry::configure_scope(|scope| {
            scope.set_tag("request_id", request_id);
        });
        Some(RequestId::new(request_id.to_string().parse().unwrap()))
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
            .set_x_request_id(MakeRequestUuidV7)
            .propagate_x_request_id();
        self.layer(tracing_service)
    }

    fn with_sentry_layer(self) -> Self {
        let sentry_service = ServiceBuilder::new()
            .layer(NewSentryLayer::new_from_top())
            .layer(SentryHttpLayer::with_transaction());
        self.layer(sentry_service)
    }
}

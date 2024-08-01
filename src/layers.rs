use axum::{
    http::{header, Request},
    Router,
};
use tower_http::trace::TraceLayer;
use uuid::Uuid;

pub trait AddLayers {
    fn add_tracing_layer(self) -> Router;
}

impl AddLayers for Router {
    fn add_tracing_layer(self) -> Self {
        self.layer(
            TraceLayer::new_for_http().make_span_with(|request: &Request<_>| {
                let request_id = Uuid::new_v4();
                let user_agent = request
                    .headers()
                    .get(header::USER_AGENT)
                    .map_or("", |h| h.to_str().unwrap_or(""));

                tracing::error_span!(
                    "http-request",
                    "http.method" = tracing::field::display(request.method()),
                    "http.uri" = tracing::field::display(request.uri()),
                    "http.version" = tracing::field::debug(request.version()),
                    "http.user_agent" = tracing::field::display(user_agent),
                    request_id = tracing::field::display(request_id),
                )
            }),
        )
    }
}

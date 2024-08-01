mod errors;
mod poem;
mod query;

use axum::http::Request;
use sqlx::SqlitePool;
use tower_http::{services::ServeDir, trace::TraceLayer};
use uuid::Uuid;

#[shuttle_runtime::main]
async fn main() -> shuttle_axum::ShuttleAxum {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();
    let db = SqlitePool::connect("sqlite://poems.sqlite3")
        .await
        .map_err(|e| shuttle_runtime::Error::Database(e.to_string()))?;
    let app = query::routes()
        .with_state(db)
        .nest_service("/static", ServeDir::new("static"))
        .layer(
            TraceLayer::new_for_http().make_span_with(|request: &Request<_>| {
                let request_id = Uuid::new_v4();
                let user_agent = request
                    .headers()
                    .get(axum::http::header::USER_AGENT)
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
        );

    Ok(app.into())

    // db.close().await;
}

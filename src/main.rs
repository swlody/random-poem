mod errors;
mod poem;
mod query;

use axum::http::{header, Request};
use sqlx::SqlitePool;
use tower_http::{services::ServeDir, trace::TraceLayer};
use tracing::Level;
use uuid::Uuid;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing subscribe
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();

    // Connect to db
    let db = SqlitePool::connect("sqlite://poems.sqlite3")
        .await
        .map_err(|e| shuttle_runtime::Error::Database(e.to_string()))?;

    // Initialize routes
    let app = query::routes()
        .with_state(db.clone())
        .nest_service("/static", ServeDir::new("static"))
        .layer(
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
        );

    // Listen and serve
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    axum::serve(listener, app).await?;

    // Cleanup db connection
    db.close().await;

    Ok(())
}

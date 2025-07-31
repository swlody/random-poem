mod api;
mod errors;
mod middleware;
mod poem;
mod site;

use std::str::FromStr as _;

use anyhow::Result;
use axum::{serve, Router};
use middleware::MakeRequestUuidV7;
use sqlx::SqlitePool;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::{
    services::ServeDir,
    trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer},
    ServiceBuilderExt as _,
};
use tracing::Level;
use tracing_subscriber::util::SubscriberInitExt as _;

use crate::errors::serve_404;

fn main() -> Result<()> {
    rubenvy::rubenvy_auto()?;

    let log_level = std::env::var("RP_LOG_LEVEL")
        .ok()
        .map_or(Level::DEBUG, |level| {
            Level::from_str(level.as_str())
                .unwrap_or_else(|_| panic!("Invalid value for RP_LOG_LEVEL: {level}"))
        });

    // Initialize tracing subscribe
    tracing_subscriber::fmt()
        .with_target(true)
        .with_max_level(log_level)
        .finish()
        .try_init()
        .expect("Unable to initialize tracing_subscriber");

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?
        .block_on(run())
}

async fn run() -> Result<()> {
    // Connect to db
    let db = SqlitePool::connect("sqlite://poems.sqlite3")
        .await
        .expect("Unable to connect to sqlite database");

    // Initialize routes
    let app = Router::new()
        .merge(site::routes())
        .nest("/api", api::routes())
        .nest_service("/static", ServeDir::new("assets/static"))
        .fallback(|| async { serve_404() })
        .with_state(db.clone())
        .layer(
            ServiceBuilder::new()
                .set_x_request_id(MakeRequestUuidV7)
                .layer(
                    TraceLayer::new_for_http()
                        .make_span_with(DefaultMakeSpan::new().include_headers(true))
                        .on_response(DefaultOnResponse::new().include_headers(true)),
                )
                .propagate_x_request_id(),
        );

    let listener = TcpListener::bind("0.0.0.0:8080").await?;
    serve(listener, app).await?;

    // Cleanup db connection
    db.close().await;

    Ok(())
}

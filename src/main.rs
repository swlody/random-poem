mod api;
mod errors;
mod layers;
mod poem;
mod render;
mod site;

use axum::{serve, Router};
use sqlx::SqlitePool;
use tokio::net::TcpListener;
use tower_http::services::{ServeDir, ServeFile};
use tracing::Level;

use crate::{errors::serve_404, layers::AddLayers as _};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing subscribe
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();

    // Connect to db
    let db = SqlitePool::connect("sqlite://poems.sqlite3").await?;

    // Initialize routes
    let app = Router::new()
        .route_service("/", ServeFile::new("static/index.html"))
        .merge(site::routes())
        .nest("/api", api::routes())
        .with_state(db.clone())
        .nest_service("/static", ServeDir::new("static"))
        .fallback(|| async { serve_404() })
        .add_tracing_layer();

    // Listen and serve
    let listener = TcpListener::bind("0.0.0.0:8080").await?;
    serve(listener, app).await?;

    // Cleanup db connection
    db.close().await;

    Ok(())
}

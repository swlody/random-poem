mod api;
mod errors;
mod layers;
mod poem;
mod render;
mod site;

use axum::Router;
use sqlx::SqlitePool;
use tower_http::services::{ServeDir, ServeFile};
use tracing::Level;

use crate::layers::AddLayers as _;

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
    let app = Router::new()
        .merge(site::routes())
        .merge(api::routes())
        .with_state(db.clone())
        .route_service("/", ServeFile::new("static/index.html"))
        .nest_service("/static", ServeDir::new("static"))
        .fallback_service(ServeFile::new("static/404.html"))
        .add_tracing_layer();

    // Listen and serve
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    axum::serve(listener, app).await?;

    // Cleanup db connection
    db.close().await;

    Ok(())
}

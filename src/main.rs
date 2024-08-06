mod api;
mod errors;
mod layers;
mod poem;
mod render;
mod site;

use axum::{serve, Router};
use secrecy::{ExposeSecret, Secret};
use sqlx::SqlitePool;
use tokio::net::TcpListener;
use tower_http::services::{ServeDir, ServeFile};
use tracing::Level;
use tracing_subscriber::{layer::SubscriberExt as _, util::SubscriberInitExt as _};

use crate::{errors::serve_404, layers::AddLayers as _};

async fn run() -> anyhow::Result<()> {
    // Initialize tracing subscribe
    tracing_subscriber::fmt()
        .with_target(true)
        .with_max_level(Level::DEBUG)
        .finish()
        .with(sentry::integrations::tracing::layer())
        .try_init()?;

    // Connect to db
    let db = SqlitePool::connect("sqlite://poems.sqlite3").await?;

    // Initialize routes
    let app = Router::new()
        .route_service("/", ServeFile::new("static/index.html"))
        .merge(site::routes())
        .nest("/api", api::routes())
        .nest_service("/static", ServeDir::new("static"))
        .fallback(|| async { serve_404() })
        .with_state(db.clone())
        .with_tracing_layer()
        .with_sentry_layer();

    // Listen and serve
    let listener = TcpListener::bind("0.0.0.0:8080").await?;
    serve(listener, app).await?;

    // Cleanup db connection
    db.close().await;

    Ok(())
}

fn main() -> anyhow::Result<()> {
    rubenvy::rubenvy(rubenvy::Environment::Development)?;

    let dsn = Secret::new(std::env::var("SENTRY_DSN")?);
    let _guard = sentry::init((
        dsn.expose_secret().as_str(),
        sentry::ClientOptions {
            release: sentry::release_name!(),
            traces_sample_rate: 1.0,
            ..Default::default()
        },
    ));

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?
        .block_on(run())
}

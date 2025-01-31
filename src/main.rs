mod api;
mod errors;
mod layers;
mod poem;
mod site;

use std::str::FromStr as _;

use anyhow::Result;
use axum::{serve, Router};
use secrecy::ExposeSecret;
use sqlx::SqlitePool;
use tokio::net::TcpListener;
use tower_http::services::ServeDir;
use tracing::Level;
use tracing_subscriber::{layer::SubscriberExt as _, util::SubscriberInitExt as _};

use crate::{errors::serve_404, layers::AddLayers as _};

struct Config {
    log_level: Option<tracing::Level>,
    trace_sample_rate: Option<f32>,
    error_sample_rate: Option<f32>,
    sentry_dsn: secrecy::SecretString,
}

impl Config {
    fn from_env() -> Self {
        Self {
            log_level: std::env::var("DAP_LOG_LEVEL").ok().map(|level| {
                Level::from_str(level.as_str())
                    .unwrap_or_else(|_| panic!("Invalid value for DAP_LOG_LEVEL: {level}"))
            }),

            trace_sample_rate: std::env::var("DAP_SENTRY_TRACING_SAMPLE_RATE")
                .ok()
                .map(|rate| {
                    let rate = rate.parse().unwrap_or_else(|_| {
                        panic!("Invalid value for DAP_SENTRY_TRACING_SAMPLE_RATE: {rate}")
                    });
                    assert!((0.0..=1.0).contains(&rate));
                    rate
                }),

            error_sample_rate: std::env::var("DAP_SENTRY_ERROR_SAMPLE_RATE")
                .ok()
                .map(|rate| {
                    let rate = rate.parse().unwrap_or_else(|_| {
                        panic!("Invalid value for DAP_SENTRY_ERROR_SAMPLE_RATE: {rate}")
                    });
                    assert!((0.0..=1.0).contains(&rate));
                    rate
                }),

            sentry_dsn: secrecy::SecretString::from(
                std::env::var("SENTRY_DSN").expect("Missing SENTRY_DSN"),
            ),
        }
    }
}

fn main() -> Result<()> {
    rubenvy::rubenvy_auto()?;

    let config = Config::from_env();

    // Initialize tracing subscribe
    tracing_subscriber::fmt()
        .with_target(true)
        .with_max_level(config.log_level.unwrap_or(Level::DEBUG))
        .finish()
        .with(sentry::integrations::tracing::layer())
        .try_init()
        .expect("Unable to initialize tracing_subscriber");

    let _guard = sentry::init((
        config.sentry_dsn.expose_secret(),
        sentry::ClientOptions {
            release: sentry::release_name!(),
            traces_sample_rate: config.trace_sample_rate.unwrap_or(0.1),
            sample_rate: config.error_sample_rate.unwrap_or(1.0),
            attach_stacktrace: true,
            ..Default::default()
        },
    ));

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
        .with_sentry_layer()
        .with_tracing_layer();

    let listener = TcpListener::bind("0.0.0.0:8080").await?;
    serve(listener, app).await?;

    // Cleanup db connection
    db.close().await;

    Ok(())
}

mod errors;
mod poem;
mod query;

use sqlx::SqlitePool;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let db = SqlitePool::connect("sqlite://poems.db").await?;
    let app = query::routes();
    let app = app.with_state(db.clone());

    let listener = tokio::net::TcpListener::bind("127.0.0.1:5050").await?;
    axum::serve(listener, app).await?;

    db.close().await;
    Ok(())
}

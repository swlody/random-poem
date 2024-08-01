mod errors;
mod poem;
mod query;

use sqlx::SqlitePool;
use tower_http::services::ServeDir;

#[shuttle_runtime::main]
async fn main() -> shuttle_axum::ShuttleAxum {
    let db = SqlitePool::connect("sqlite://poems.db")
        .await
        .map_err(|e| shuttle_runtime::Error::Database(e.to_string()))?;
    let app = query::routes()
        .with_state(db)
        .nest_service("/static", ServeDir::new("static"));

    Ok(app.into())

    // db.close().await;
}

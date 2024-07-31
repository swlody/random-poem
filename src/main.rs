mod errors;
mod poem;
mod query;

use sqlx::SqlitePool;

#[shuttle_runtime::main]
async fn main() -> shuttle_axum::ShuttleAxum {
    let db = SqlitePool::connect("sqlite://poems.db")
        .await
        .map_err(|e| shuttle_runtime::Error::Database(e.to_string()))?;
    let app = query::routes().with_state(db);

    Ok(app.into())

    // db.close().await;
}

use axum::{
    body::Body,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use thiserror::Error;

#[derive(Serialize, Deserialize, Clone)]
struct Poem {
    pub title: String,
    pub author: String,
    pub content: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let db = SqlitePool::connect("sqlite://poems.db").await?;
    let app = Router::new()
        .route("/random", get(random))
        .route("/random/author/:author", get(random_by_author))
        .with_state(db.clone());

    let listener = tokio::net::TcpListener::bind("127.0.0.1:5050").await?;
    axum::serve(listener, app).await?;

    db.close().await;
    Ok(())
}

#[derive(Error, Debug)]
enum Error {
    #[error(transparent)]
    DatabaseError(#[from] sqlx::Error),
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, Body::empty()).into_response()
    }
}

type Result<T> = std::result::Result<T, Error>;

async fn random(State(db): State<SqlitePool>) -> Result<Response> {
    let poem = sqlx::query_as!(Poem, "SELECT * FROM poems ORDER BY RANDOM()")
        .fetch_one(&db)
        .await?;
    Ok((StatusCode::OK, Json(poem)).into_response())
}

async fn random_by_author(
    Path(author): Path<String>,
    State(db): State<SqlitePool>,
) -> Result<Response> {
    let author = author.replace('_', " ");
    let poem = sqlx::query_as!(
        Poem,
        "SELECT * FROM poems WHERE author = $1 ORDER BY RANDOM()",
        author
    )
    .fetch_one(&db)
    .await?;

    Ok((StatusCode::OK, Json(poem)).into_response())
}

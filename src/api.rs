use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse as _, Response},
    routing::get,
    Json, Router,
};
use sqlx::SqlitePool;

use crate::{errors::Result, poem::Poem};

async fn api_random(State(db): State<SqlitePool>) -> Result<Response> {
    let poem = Poem::get_random(db).await?;
    Ok((StatusCode::OK, Json(poem)).into_response())
}

async fn api_random_by_author(
    Path(author): Path<String>,
    State(db): State<SqlitePool>,
) -> Result<Response> {
    let poem = Poem::get_random_by_author(&author, db).await?;
    Ok((StatusCode::OK, Json(poem)).into_response())
}

async fn api_specific_poem(
    Path((author, title)): Path<(String, String)>,
    State(db): State<SqlitePool>,
) -> Result<Response> {
    let poem = Poem::get_specific_poem(&author, &title, db).await?;
    Ok((StatusCode::OK, Json(poem)).into_response())
}

pub fn routes() -> Router<SqlitePool> {
    Router::new()
        .route("/api/poem/:author/:title", get(api_specific_poem))
        .route("/api/poem/random", get(api_random))
        .route("/api/poem/:author/random", get(api_random_by_author))
}

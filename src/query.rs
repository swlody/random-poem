use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse as _, Response},
    routing::get,
    Json, Router,
};
use maud::Markup;
use sqlx::SqlitePool;

use crate::{errors::Result, poem::Poem};

async fn api_random(State(db): State<SqlitePool>) -> Result<Response> {
    let poem = Poem::get_random(db).await?;
    Ok((StatusCode::OK, Json(poem)).into_response())
}

async fn html_random(State(db): State<SqlitePool>) -> Result<Markup> {
    let poem = Poem::get_random(db).await?;
    Ok(poem.into_html())
}

async fn api_random_by_author(
    Path(author): Path<String>,
    State(db): State<SqlitePool>,
) -> Result<Response> {
    let poem = Poem::get_random_by_author(&author, db).await?;
    Ok((StatusCode::OK, Json(poem)).into_response())
}

async fn html_random_by_author(
    Path(author): Path<String>,
    State(db): State<SqlitePool>,
) -> Result<Markup> {
    let poem = Poem::get_random_by_author(&author, db).await?;
    Ok(poem.into_html())
}

async fn api_specific_poem(
    Path((author, title)): Path<(String, String)>,
    State(db): State<SqlitePool>,
) -> Result<Response> {
    let poem = Poem::get_specific_poem(&author, &title, db).await?;
    Ok((StatusCode::OK, Json(poem)).into_response())
}

async fn html_specific_poem(
    Path((author, title)): Path<(String, String)>,
    State(db): State<SqlitePool>,
) -> Result<Markup> {
    let poem = Poem::get_specific_poem(&author, &title, db).await?;
    Ok(poem.into_html())
}

pub fn routes() -> Router<SqlitePool> {
    Router::new()
        .route("/author/:author/title/:title", get(html_specific_poem))
        .route("/random", get(html_random))
        .route("/random/:author", get(html_random_by_author))
        .route("/api/author/:author/title/:title", get(api_specific_poem))
        .route("/api/random", get(api_random))
        .route("/api/random/:author", get(api_random_by_author))
}

#[cfg(test)]
mod tests {
    use axum::{body::Body, http::Request};
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    use super::*;

    #[tokio::test]
    async fn can_get_random_poem() -> anyhow::Result<()> {
        let db = SqlitePool::connect("sqlite://poems.db").await?;
        let app = routes().with_state(db);
        let response = app
            .oneshot(Request::builder().uri("/api/random").body(Body::empty())?)
            .await?;
        assert_eq!(StatusCode::OK, response.status());
        assert!(!response.into_body().collect().await?.to_bytes().is_empty());
        Ok(())
    }
}

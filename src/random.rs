use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse as _, Response},
    routing::get,
    Json, Router,
};
use sqlx::SqlitePool;

use crate::{errors::Result, poem::Poem};

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

pub fn routes() -> Router<SqlitePool> {
    Router::new()
        .route("/random", get(random))
        .route("/random/:author", get(random_by_author))
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
            .oneshot(Request::builder().uri("/random").body(Body::empty())?)
            .await?;
        assert_eq!(StatusCode::OK, response.status());
        assert!(!response.into_body().collect().await?.to_bytes().is_empty());
        Ok(())
    }
}

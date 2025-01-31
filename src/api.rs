use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Router,
};
use sqlx::SqlitePool;

use crate::{errors::Result, poem::Poem};

#[tracing::instrument]
async fn random_poem(State(db): State<SqlitePool>) -> Result<impl IntoResponse> {
    Poem::random(db).await?.into_json()
}

#[tracing::instrument]
async fn random_poem_by_author(
    Path(author): Path<String>,
    State(db): State<SqlitePool>,
) -> Result<impl IntoResponse> {
    Poem::random_by_author(&author, db).await?.into_json()
}

#[tracing::instrument]
async fn specific_poem(
    Path((author, title)): Path<(String, String)>,
    State(db): State<SqlitePool>,
) -> Result<impl IntoResponse> {
    Poem::from_author_and_title(&author, &title, db)
        .await?
        .into_json()
}

#[tracing::instrument]
async fn health(State(db): State<SqlitePool>) -> StatusCode {
    if db.is_closed() {
        StatusCode::INTERNAL_SERVER_ERROR
    } else {
        StatusCode::OK
    }
}

pub fn routes() -> Router<SqlitePool> {
    Router::new()
        .route("/health", get(health))
        .route("/poem/{author}/{title}", get(specific_poem))
        .route("/poem/random", get(random_poem))
        .route("/poem/{author}/random", get(random_poem_by_author))
}

#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    use super::*;

    #[tokio::test]
    async fn can_get_random_poem() -> anyhow::Result<()> {
        let db = SqlitePool::connect("sqlite://poems.sqlite3").await?;
        let app = routes().with_state(db.clone());

        let response = app
            .oneshot(Request::get("/poem/random").body(Body::empty())?)
            .await?;

        assert_eq!(StatusCode::OK, response.status());
        assert!(!response.into_body().collect().await?.to_bytes().is_empty());

        db.close().await;
        Ok(())
    }

    #[tokio::test]
    async fn can_get_specific_poem() -> anyhow::Result<()> {
        let db = SqlitePool::connect("sqlite://poems.sqlite3").await?;
        let app = routes().with_state(db.clone());

        let response = app
            .oneshot(Request::get("/poem/Edgar%20Allan%20Poe/The%20Raven").body(Body::empty())?)
            .await?;

        assert_eq!(StatusCode::OK, response.status());
        let body = response.into_body().collect().await?.to_bytes();
        let json = serde_json::from_str::<Poem>(std::str::from_utf8(&body)?)?;
        insta::assert_json_snapshot!(json);

        Ok(())
    }
}

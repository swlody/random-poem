use axum::{
    extract::{Path, State},
    response::{Html, IntoResponse, Redirect},
    routing::get,
    Router,
};
use rinja::Template;
use sqlx::SqlitePool;
use urlencoding::encode;

use crate::{errors::Result, poem::Poem};

#[tracing::instrument]
async fn index() -> Result<impl IntoResponse> {
    #[derive(Template)]
    #[template(path = "index.html")]
    struct IndexTemplate;

    Ok(Html(IndexTemplate.render()?))
}

// Maybe replace encoding for redirects:
// Space: "%20" -> " "
// Colon: "%3A" -> ":"
// Ampersand: "%26" -> "&"
// Command: "%2C" -> ","

#[tracing::instrument]
async fn random_poem(State(db): State<SqlitePool>) -> Result<impl IntoResponse> {
    let Poem { author, title, .. } = Poem::random(db).await?;
    Ok(Redirect::to(&format!(
        "/poem/{}/{}",
        encode(&author),
        encode(&title)
    )))
}

#[tracing::instrument]
async fn random_poem_by_author(
    Path(author): Path<String>,
    State(db): State<SqlitePool>,
) -> Result<impl IntoResponse> {
    let Poem { author, title, .. } = Poem::random_by_author(&author, db).await?;
    Ok(Redirect::to(&format!(
        "/poem/{}/{}",
        encode(&author),
        encode(&title)
    )))
}

#[tracing::instrument]
async fn specific_poem(
    Path((author, title)): Path<(String, String)>,
    State(db): State<SqlitePool>,
) -> Result<impl IntoResponse> {
    Poem::from_author_and_title(&author, &title, db)
        .await?
        .into_html()
}

#[tracing::instrument]
async fn author_landing(
    Path(author): Path<String>,
    State(db): State<SqlitePool>,
) -> Result<impl IntoResponse> {
    #[derive(Template)]
    #[template(path = "author.html")]
    struct AuthorTemplate {
        author: String,
    }

    // Check author exists - DB error will return a 404
    let _ = Poem::random_by_author(&author, db).await?;

    Ok(Html(AuthorTemplate { author }.render()?))
}

#[tracing::instrument]
async fn poem_of_the_day(State(db): State<SqlitePool>) -> Result<impl IntoResponse> {
    Poem::poem_of_the_day(db).await?.into_html()
}

pub fn routes() -> Router<SqlitePool> {
    Router::new()
        .route("/", get(index))
        .route("/poem/{author}/{title}", get(specific_poem))
        .route("/poem/random", get(random_poem))
        .route("/poem/{author}/random", get(random_poem_by_author))
        .route("/poet/{author}", get(author_landing))
    // .route("/today", get(poem_of_the_day))
}

#[cfg(test)]
mod tests {
    use std::str;

    use anyhow::Context as _;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use http_body_util::BodyExt;
    use tower::{Service, ServiceExt};

    use super::*;

    #[tokio::test]
    async fn can_get_random_poem() -> anyhow::Result<()> {
        let db = SqlitePool::connect("sqlite://poems.sqlite3").await?;
        let mut app = routes().with_state(db.clone());

        // First call random URL to get redirect location
        let response = app
            .call(Request::get("/poem/random").body(Body::empty())?)
            .await?;

        assert_eq!(StatusCode::SEE_OTHER, response.status());
        // Convert redirect to string, this could fail if it's not encoded nicely
        let redirect = response
            .headers()
            .get("location")
            .context("Failed to get redirect location")?
            .to_str()
            .with_context(|| format!("{:?}", response))?
            .to_owned();
        assert!(response.into_body().collect().await?.to_bytes().is_empty());

        // Follow redirect to ensure poem actually exists
        let response = app
            .call(Request::get(&redirect).body(Body::empty())?)
            .await?;
        assert_eq!(StatusCode::OK, response.status());
        if response.status() != StatusCode::OK {
            anyhow::bail!("Failed to fetch poem from redirect: {redirect}");
        }
        assert!(!response.into_body().collect().await?.to_bytes().is_empty());

        db.close().await;
        Ok(())
    }

    #[tokio::test]
    async fn can_get_specific_poem() -> anyhow::Result<()> {
        let db = SqlitePool::connect("sqlite://poems.sqlite3").await?;
        let app = crate::layers::AddLayers::with_tracing_layer(routes().with_state(db.clone()));

        let response = app
            .oneshot(Request::get("/poem/Edgar%20Allan%20Poe/The%20Raven").body(Body::empty())?)
            .await?;

        assert_eq!(StatusCode::OK, response.status());
        let body = response.into_body().collect().await?.to_bytes();
        insta::assert_snapshot!(str::from_utf8(&body)?);

        Ok(())
    }
}

use axum::{
    extract::{Path, State},
    response::{IntoResponse as _, Redirect, Response},
    routing::get,
    Router,
};
use maud::{html, Markup};
use sqlx::SqlitePool;
use urlencoding::encode;

use crate::{errors::Result, poem::Poem, render::wrap_body};

async fn random(State(db): State<SqlitePool>) -> Result<Response> {
    let Poem { author, title, .. } = Poem::random(db).await?;
    let author = encode(&author);
    let title = encode(&title);
    Ok(Redirect::to(&format!("/poem/{author}/{title}")).into_response())
}

async fn random_by_author(
    Path(author): Path<String>,
    State(db): State<SqlitePool>,
) -> Result<Response> {
    let Poem { author, title, .. } = Poem::random_by_author(&author, db).await?;
    let author = encode(&author);
    let title = encode(&title);
    Ok(Redirect::to(&format!("/poem/{author}/{title}")).into_response())
}

async fn specific_poem(
    Path((author, title)): Path<(String, String)>,
    State(db): State<SqlitePool>,
) -> Result<Markup> {
    let poem = Poem::from_author_and_title(&author, &title, db).await?;
    let body = wrap_body(&poem.into_html());
    Ok(body)
}

async fn author_landing(
    Path(author): Path<String>,
    State(db): State<SqlitePool>,
) -> Result<Markup> {
    // Check author exists
    Poem::random_by_author(&author, db).await?;

    let body = wrap_body(&html! {
        div id = "body-content" {
            a href = (format!("/poem/{author}/random")) {
                "Click here for a random poem by " (author)
            }
        }
    });
    Ok(body)
}

pub fn routes() -> Router<SqlitePool> {
    Router::new()
        .route("/poem/:author/:title", get(specific_poem))
        .route("/poem/random", get(random))
        .route("/poem/:author/random", get(random_by_author))
        .route("/poet/:author", get(author_landing))
}

#[cfg(test)]
mod tests {
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
        let response = app
            .call(Request::get("/poem/random").body(Body::empty())?)
            .await?;
        assert_eq!(StatusCode::SEE_OTHER, response.status());
        let redirect = response
            .headers()
            .get("location")
            .context("Failed to get redirect location")?
            .to_str()
            .with_context(|| format!("{:?}", response))?
            .to_owned();
        assert!(response.into_body().collect().await?.to_bytes().is_empty());
        dbg!(&redirect);

        // Follow redirect to ensure poem actually exists:
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
        let app = crate::layers::AddLayers::add_tracing_layer(routes().with_state(db.clone()));
        let response = app
            .oneshot(Request::get("/poem/Edgar%20Allan%20Poe/The%20Raven").body(Body::empty())?)
            .await?;
        assert_eq!(StatusCode::OK, response.status());
        let body = response.into_body().collect().await?.to_bytes();
        let s = std::str::from_utf8(&body)?;
        insta::assert_snapshot!(s);
        Ok(())
    }
}

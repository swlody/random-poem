use axum::{
    extract::{Path, State},
    response::{IntoResponse as _, Redirect, Response},
    routing::get,
    Router,
};
use maud::Markup;
use sqlx::SqlitePool;

use crate::{errors::Result, poem::Poem, render::render_body};

async fn html_random(State(db): State<SqlitePool>) -> Result<Response> {
    let poem = Poem::get_random(db).await?;
    // TODO don't do this twice
    let author = poem.author.replace(' ', "_");
    let title = poem.title.replace(' ', "_");
    Ok(Redirect::to(&format!("/poem/{author}/{title}")).into_response())
}

async fn html_random_by_author(
    Path(author): Path<String>,
    State(db): State<SqlitePool>,
) -> Result<Response> {
    let poem = Poem::get_random_by_author(&author, db).await?;
    let title = poem.title.replace(' ', "_");
    Ok(Redirect::to(&format!("/poem/{author}/{title}")).into_response())
}

async fn html_specific_poem(
    Path((author, title)): Path<(String, String)>,
    State(db): State<SqlitePool>,
) -> Result<Markup> {
    let poem = Poem::get_specific_poem(&author, &title, db).await?;
    let body = render_body(poem.into_html());
    Ok(body)
}

pub fn routes() -> Router<SqlitePool> {
    Router::new()
        .route("/poem/:author/:title", get(html_specific_poem))
        .route("/poem/random", get(html_random))
        .route("/poem/:author/random", get(html_random_by_author))
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
        let app = routes().with_state(db);
        let response = app
            .oneshot(Request::builder().uri("/api/random").body(Body::empty())?)
            .await?;
        assert_eq!(StatusCode::OK, response.status());
        assert!(!response.into_body().collect().await?.to_bytes().is_empty());
        Ok(())
    }
}

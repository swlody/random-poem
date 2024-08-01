use axum::{
    extract::{Path, State},
    response::{IntoResponse as _, Redirect, Response},
    routing::get,
    Router,
};
use maud::{html, Markup};
use sqlx::SqlitePool;

use crate::{errors::Result, poem::Poem, render::wrap_body};

async fn random(State(db): State<SqlitePool>) -> Result<Response> {
    let Poem { author, title, .. } = Poem::get_random(db).await?;
    Ok(Redirect::to(&format!("/poem/{author}/{title}")).into_response())
}

async fn random_by_author(
    Path(author): Path<String>,
    State(db): State<SqlitePool>,
) -> Result<Response> {
    let Poem { author, title, .. } = Poem::get_random_by_author(&author, db).await?;
    Ok(Redirect::to(&format!("/poem/{author}/{title}")).into_response())
}

async fn specific_poem(
    Path((author, title)): Path<(String, String)>,
    State(db): State<SqlitePool>,
) -> Result<Markup> {
    let poem = Poem::get_specific_poem(&author, &title, db).await?;
    let body = wrap_body(&poem.into_html());
    Ok(body)
}

async fn author_landing(
    Path(author): Path<String>,
    State(db): State<SqlitePool>,
) -> Result<Markup> {
    // Check author exists
    sqlx::query!("SELECT * FROM poems WHERE author = $1", author)
        .fetch_one(&db)
        .await?;

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

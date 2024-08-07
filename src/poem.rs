use maud::{html, Markup};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

use crate::errors::Result;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Poem {
    pub title: String,
    pub author: String,
    pub content: String,
}

impl Poem {
    #[tracing::instrument]
    pub async fn random(db: SqlitePool) -> Result<Self> {
        // TODO this is obviously weighted towards more prolific poets,
        // maybe we should get a random poet and then a random poem from that poet?
        let poem = sqlx::query_as!(Self, "SELECT * FROM poems ORDER BY RANDOM()")
            .fetch_one(&db)
            .await?;
        Ok(poem)
    }

    #[tracing::instrument]
    pub async fn random_by_author(author: &str, db: SqlitePool) -> Result<Self> {
        let poem = sqlx::query_as!(
            Self,
            "SELECT * FROM poems WHERE author = $1 ORDER BY RANDOM()",
            author
        )
        .fetch_one(&db)
        .await?;
        Ok(poem)
    }

    #[tracing::instrument]
    pub async fn from_author_and_title(author: &str, title: &str, db: SqlitePool) -> Result<Self> {
        let poem = sqlx::query_as!(
            Self,
            "SELECT * FROM poems WHERE author = $1 AND title = $2",
            author,
            title
        )
        .fetch_one(&db)
        .await?;
        Ok(poem)
    }

    #[tracing::instrument]
    pub fn into_html(self) -> Markup {
        html! {
            div id = "poem" {
                h1 id = "poem-title" {
                    (self.title)
                }
                h3 id = "poem-author" {
                    "By " a href = (format!("/poet/{}", self.author)) {
                        (self.author)
                    }
                }
                p id = "poem-content" {
                    @for line in self.content.lines() {
                        (line)
                        br;
                    }
                }
            }
        }
    }
}

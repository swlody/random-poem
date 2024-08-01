use maud::{html, Markup};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

use crate::errors::Result;

#[derive(Serialize, Deserialize, Clone)]
pub struct Poem {
    pub title: String,
    pub author: String,
    pub content: String,
}

impl Poem {
    pub async fn get_random(db: SqlitePool) -> Result<Self> {
        let poem = sqlx::query_as!(Self, "SELECT * FROM poems ORDER BY RANDOM()")
            .fetch_one(&db)
            .await?;
        Ok(poem)
    }

    pub async fn get_random_by_author(author: &str, db: SqlitePool) -> Result<Self> {
        let author = author.replace('_', " ");
        let poem = sqlx::query_as!(
            Self,
            "SELECT * FROM poems WHERE author = $1 ORDER BY RANDOM()",
            author
        )
        .fetch_one(&db)
        .await?;
        Ok(poem)
    }

    pub async fn get_specific_poem(author: &str, title: &str, db: SqlitePool) -> Result<Self> {
        // TODO middleware to replace _ with spaces
        let author = author.replace('_', " ");
        let title = title.replace('_', " ");
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

    pub fn into_html(&self) -> Markup {
        html! {
            div id = "poem" {
                h1 id = "poem-title" {
                    (self.title)
                }
                h2 id = "poem-author" {
                    "By " (self.author)
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

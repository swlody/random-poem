use chrono::{Datelike, Utc};
use maud::{html, Markup};
use rand::{Rng, SeedableRng as _};
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
    pub async fn poem_of_the_day(db: SqlitePool) -> Result<Self> {
        let today = Utc::now().num_days_from_ce();
        let days = u64::try_from(today).expect("System clock is very confused");
        let mut rng = rand::rngs::SmallRng::seed_from_u64(days);
        let random_choice = rng.gen_range(0.0..=1.0);

        // TODO fairness in ORDERING
        // TODO test performance vs alternate query
        let poem = sqlx::query_as!(
            Self,
            r#"
            WITH author_counts AS (
                SELECT author, COUNT(*) as poem_count
                FROM poems
                GROUP BY author
            ),
            weighted_poems AS (
                SELECT p.author, p.title, p.content, 1.0 / ac.poem_count as weight
                FROM poems p
                JOIN author_counts ac ON p.author = ac.author
            )
            SELECT author, title, content
            FROM weighted_poems
            WHERE $1 <= weight
            "#,
            random_choice
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

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_rng() -> anyhow::Result<()> {
        // TODO deterministic testing
        // let db = SqlitePool::connect("sqlite://poems.sqlite3").await?;
        // let poem = Poem::poem_of_the_day(db).await?;
        // dbg!(poem);
        Ok(())
    }
}

use std::{collections::HashMap, fs::File, io::Read as _, sync::Arc};

use axum::{extract::State, http::StatusCode, routing::get, Json, Router};
use chrono::NaiveDate;
use rand::Rng as _;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
struct Poem {
    pub title: String,
    pub author: String,
    pub date: Option<NaiveDate>,
    pub content: String,
}

#[derive(Clone)]
struct AppState {
    poems: Arc<HashMap<String, Poem>>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let mut poems = HashMap::<String, Poem>::new();
    let mut file = File::open("poems.json")?;
    let mut data = String::new();
    file.read_to_string(&mut data)?;
    let json: Vec<Poem> = serde_json::from_str(&data)?;
    for poem in json {
        poems.insert(poem.title.clone(), poem);
    }
    let app = Router::new()
        .route("/random", get(random))
        .with_state(AppState {
            poems: Arc::new(poems),
        });

    let listener = tokio::net::TcpListener::bind("127.0.0.1:5050").await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn random(State(state): State<AppState>) -> (StatusCode, Json<Poem>) {
    let mut rng = rand::thread_rng();
    let random_index = rng.gen_range(0..state.poems.len());
    let mut current_index = 0;

    let mut random_poem: Option<Poem> = None;
    for value in state.poems.values() {
        if current_index == random_index {
            random_poem = Some(value.clone());
            break;
        }
        current_index += 1;
    }

    (axum::http::StatusCode::OK, Json(random_poem.unwrap()))
}

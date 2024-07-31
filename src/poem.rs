use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Poem {
    pub title: String,
    pub author: String,
    pub content: String,
}

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Container {
    pub name: String,
    pub path: String,
    pub apps: Vec<String>,
}

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Command {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub command: String,
    pub args: Vec<String>,
}

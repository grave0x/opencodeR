use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectID(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectInfo {
    pub id: ProjectID,
    pub name: String,
    pub path: String,
    pub description: Option<String>,
}

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connection {
    pub id: String,
    pub name: String,
    pub kind: ConnectionKind,
    pub config: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConnectionKind {
    #[serde(rename = "mcp")]
    Mcp,
    #[serde(rename = "acp")]
    Acp,
    #[serde(rename = "custom")]
    Custom,
}

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Integration {
    pub id: String,
    pub name: String,
    pub kind: IntegrationKind,
    pub config: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IntegrationKind {
    #[serde(rename = "github")]
    GitHub,
    #[serde(rename = "gitlab")]
    GitLab,
    #[serde(rename = "slack")]
    Slack,
    #[serde(rename = "custom")]
    Custom,
}

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationRef(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationInfo {
    pub id: LocationRef,
    pub name: Option<String>,
    pub workspace_id: Option<String>,
    pub kind: LocationKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LocationKind {
    #[serde(rename = "local")]
    Local,
    #[serde(rename = "remote")]
    Remote,
    #[serde(rename = "cloud")]
    Cloud,
}

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevertState {
    pub kind: RevertKind,
    pub checkpoint_id: String,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RevertKind {
    #[serde(rename = "file")]
    File,
    #[serde(rename = "session")]
    Session,
}

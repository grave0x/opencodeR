use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSystemInfo {
    pub id: String,
    pub path: String,
    pub kind: FileSystemKind,
    pub writable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FileSystemKind {
    #[serde(rename = "local")]
    Local,
    #[serde(rename = "remote")]
    Remote,
}

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reference {
    pub id: String,
    pub kind: ReferenceKind,
    pub path: String,
    pub range: Option<ReferenceRange>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReferenceKind {
    #[serde(rename = "file")]
    File,
    #[serde(rename = "symbol")]
    Symbol,
    #[serde(rename = "web")]
    Web,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferenceRange {
    pub start: Position,
    pub end: Position,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub line: u32,
    pub column: u32,
}

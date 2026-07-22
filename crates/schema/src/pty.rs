use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PtyInfo {
    pub id: String,
    pub cols: u32,
    pub rows: u32,
    pub pid: Option<u32>,
}

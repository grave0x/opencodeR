use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PtyTicket {
    pub id: String,
    pub pty_id: String,
    pub token: String,
    pub expires_at: i64,
}

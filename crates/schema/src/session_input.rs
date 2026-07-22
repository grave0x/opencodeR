use serde::{Deserialize, Serialize};
use super::session_id::SessionID;
use super::prompt::PromptInput;
use super::schema::DateTimeUtcFromMillis;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInput {
    pub id: SessionInputID,
    pub session_id: SessionID,
    pub prompt: PromptInput,
    pub status: InputStatus,
    pub created_at: DateTimeUtcFromMillis,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInputID(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InputStatus {
    #[serde(rename = "pending")]
    Pending,
    #[serde(rename = "processing")]
    Processing,
    #[serde(rename = "completed")]
    Completed,
    #[serde(rename = "failed")]
    Failed,
}

use serde::{Deserialize, Serialize};
use super::schema::DateTimeUtcFromMillis;
use super::session_id::SessionID;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionEvent {
    pub id: String,
    pub session_id: SessionID,
    pub kind: SessionEventKind,
    pub data: serde_json::Value,
    pub timestamp: DateTimeUtcFromMillis,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionEventKind {
    #[serde(rename = "message_added")]
    MessageAdded,
    #[serde(rename = "message_updated")]
    MessageUpdated,
    #[serde(rename = "tool_call")]
    ToolCall,
    #[serde(rename = "tool_result")]
    ToolResult,
    #[serde(rename = "session_created")]
    SessionCreated,
    #[serde(rename = "session_archived")]
    SessionArchived,
}

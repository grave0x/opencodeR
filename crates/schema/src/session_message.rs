use serde::{Deserialize, Serialize};
use super::session_id::SessionID;
use super::schema::DateTimeUtcFromMillis;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionMessageID(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMessage {
    pub id: SessionMessageID,
    pub session_id: SessionID,
    pub role: MessageRole,
    pub content: Vec<MessageContent>,
    pub created_at: DateTimeUtcFromMillis,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageRole {
    #[serde(rename = "user")]
    User,
    #[serde(rename = "assistant")]
    Assistant,
    #[serde(rename = "system")]
    System,
    #[serde(rename = "tool")]
    Tool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageContent {
    Text { text: String },
    ToolCall {
        id: String,
        name: String,
        arguments: serde_json::Value,
    },
    ToolResult {
        id: String,
        content: String,
    },
}

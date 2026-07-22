use opencode_schema::agent::AgentID;
use opencode_schema::model::ModelRef;
use opencode_schema::permission::PermissionAction;
use opencode_schema::pty_ticket::PtyTicket;
use opencode_schema::revert::RevertState;
use opencode_schema::session_event::SessionEvent;
use opencode_schema::session_id::SessionID;
use opencode_schema::session_message::{SessionMessage, SessionMessageID};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type LocationResponse<T> = T;

// ---- Generic wrappers ----

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataResponse<T> {
    pub data: T,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CursorResponse<T> {
    pub data: T,
    pub cursor: CursorLinks,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CursorLinks {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoContent;

// ---- Health ----

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    pub healthy: bool,
}

// ---- Session ----

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionCreateInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<SessionID>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent: Option<AgentID>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<ModelRef>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionActiveMap {
    #[serde(flatten)]
    pub sessions: HashMap<SessionID, SessionActive>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionActive {
    pub r#type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSwitchAgentInput {
    pub agent: AgentID,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSwitchModelInput {
    pub model: ModelRef,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionPromptInput {
    pub prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<SessionMessageID>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delivery: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resume: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionRevertStageInput {
    pub message_id: SessionMessageID,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub files: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevertStageResponse {
    pub data: RevertState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionContextResponse {
    pub data: Vec<SessionMessage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionHistoryResponse {
    pub data: Vec<SessionEvent>,
    pub has_more: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionCreatePermissionInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub action: PermissionAction,
    pub resources: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub save: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent: Option<AgentID>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionCreateResponse {
    pub id: String,
    pub effect: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionPermissionReplyInput {
    pub reply: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

// ---- Question ----

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionReplyInput {
    pub answer: String,
}

// ---- PTY ----

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PtyCreateInput {
    pub cols: u32,
    pub rows: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PtyUpdateInput {
    pub cols: u32,
    pub rows: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PtyConnectTokenResponse {
    pub data: PtyTicket,
}

// ---- Credential ----

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialUpdateInput {
    pub label: String,
}

// ---- Integration ----

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationConnectKeyInput {
    pub key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationConnectOAuthInput {
    pub method_id: String,
    pub inputs: HashMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationAttemptCompleteInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
}

// ---- Project Copy ----

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectCopyCreateInput {
    pub source_directory: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectCopyRemoveInput {
    pub source_directory: String,
}

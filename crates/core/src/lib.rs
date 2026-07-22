// Core service trait definitions
pub mod memory;
pub mod discovery;

use opencode_r_schema::agent::AgentInfo;
use opencode_r_schema::command::Command;
use opencode_r_schema::event::Event;
use opencode_r_schema::integration::Integration;
use opencode_r_schema::model::ModelInfo;
use opencode_r_schema::permission::PermissionAction;
use opencode_r_schema::provider::ProviderInfo;
use opencode_r_schema::pty::PtyInfo;
use opencode_r_schema::pty_ticket::PtyTicket;
use opencode_r_schema::question::Question;
use opencode_r_schema::reference::Reference;
use opencode_r_schema::revert::RevertState;
use opencode_r_schema::schema::{AbsolutePath, RelativePath};
use opencode_r_schema::session::SessionInfo;
use opencode_r_schema::session_event::SessionEvent;
use opencode_r_schema::session_id::SessionID;
use opencode_r_schema::session_message::{SessionMessage, SessionMessageID};
use opencode_r_schema::skill::Skill;
use opencode_r_schema::workspace::Workspace;
use std::collections::HashMap;

// ---- Agent Service ----

pub trait AgentService: Send + Sync {
    fn list(&self) -> Vec<AgentInfo>;
}

// ---- Catalog Service (models + providers) ----

pub trait CatalogService: Send + Sync {
    fn list_models(&self) -> Vec<ModelInfo>;
    fn list_providers(&self) -> Vec<ProviderInfo>;
    fn get_provider(&self, id: &str) -> Option<ProviderInfo>;
}

// ---- Session Service ----

pub type SessionListResult = Vec<SessionInfo>;

pub struct SessionListQuery {
    pub workspace: Option<Workspace>,
    pub limit: Option<u32>,
    pub order: Option<String>,
    pub search: Option<String>,
    pub directory: Option<AbsolutePath>,
    pub project: Option<String>,
    pub subpath: Option<RelativePath>,
    pub cursor: Option<String>,
    pub cursor_id: Option<String>,
    pub cursor_time: Option<i64>,
    pub cursor_direction: Option<String>,
}

pub struct SessionCreateInput {
    pub id: Option<SessionID>,
    pub agent: Option<String>,
    pub model: Option<String>,
    pub location: Option<String>,
}

pub struct SessionPromptInput {
    pub id: Option<SessionMessageID>,
    pub prompt: String,
    pub delivery: Option<String>,
    pub resume: Option<bool>,
}

pub struct SessionSwitchAgentInput {
    pub agent: String,
}

pub struct SessionSwitchModelInput {
    pub model: String,
}

pub struct SessionRevertStageInput {
    pub message_id: SessionMessageID,
    pub files: Option<bool>,
}

pub struct SessionHistoryQuery {
    pub limit: Option<u32>,
    pub after: Option<u32>,
}

pub struct SessionMessagesQuery {
    pub session_id: SessionID,
    pub limit: Option<u32>,
    pub order: Option<String>,
    pub cursor: Option<String>,
}

pub struct SessionHistoryResult {
    pub events: Vec<SessionEvent>,
    pub has_more: bool,
}

pub trait SessionService: Send + Sync {
    fn list(&self, query: SessionListQuery) -> SessionListResult;
    fn create(&self, input: SessionCreateInput) -> SessionInfo;
    fn active(&self) -> HashMap<SessionID, String>;
    fn get(&self, id: &SessionID) -> Option<SessionInfo>;
    fn switch_agent(&self, session_id: &SessionID, agent: &str) -> Result<(), ()>;
    fn switch_model(&self, session_id: &SessionID, model: &str) -> Result<(), ()>;
    fn prompt(&self, session_id: &SessionID, input: SessionPromptInput) -> Result<String, String>;
    fn compact(&self, session_id: &SessionID) -> Result<(), String>;
    fn delete(&self, session_id: &SessionID) -> Result<(), ()>;
    fn wait(&self, session_id: &SessionID) -> Result<(), String>;
    fn revert_stage(&self, session_id: &SessionID, input: SessionRevertStageInput) -> Result<RevertState, String>;
    fn revert_clear(&self, session_id: &SessionID) -> Result<(), String>;
    fn revert_commit(&self, session_id: &SessionID) -> Result<(), String>;
    fn context(&self, session_id: &SessionID) -> Result<Vec<SessionMessage>, String>;
    fn history(&self, session_id: &SessionID, query: SessionHistoryQuery) -> Result<SessionHistoryResult, String>;
    fn events(&self, session_id: &SessionID, after: Option<u32>) -> Vec<SessionEvent>;
    fn global_events(&self, after: Option<u32>, limit: Option<u32>) -> Vec<SessionEvent>;
    fn subscribe_events(&self) -> tokio::sync::broadcast::Receiver<SessionEvent>;
    fn cost_summary(&self) -> opencode_r_schema::session::CostSummary;
    fn cost_breakdown(&self, session_id: &SessionID) -> Option<opencode_r_schema::session::CostBreakdown>;
    fn pause(&self, session_id: &SessionID) -> Result<(), ()>;
    fn resume(&self, session_id: &SessionID) -> Result<(), ()>;
    fn freeze(&self, session_id: &SessionID) -> Result<(), ()>;
    fn terminate(&self, session_id: &SessionID) -> Result<(), ()>;
    fn set_group(&self, session_id: &SessionID, group: Option<String>) -> Result<(), ()>;
    fn list_groups(&self) -> Vec<(String, usize)>;
    fn interrupt(&self, session_id: &SessionID);
    fn messages(&self, query: SessionMessagesQuery) -> Result<Vec<SessionMessage>, String>;
    fn message(&self, session_id: &SessionID, message_id: &SessionMessageID) -> Option<SessionMessage>;
}

// ---- PTY Service ----

pub struct PtyCreateInput {
    pub cols: u32,
    pub rows: u32,
    pub cwd: Option<String>,
    pub command: Option<String>,
}

pub struct PtyUpdateInput {
    pub cols: u32,
    pub rows: u32,
}

pub trait PtyService: Send + Sync {
    fn list(&self) -> Vec<PtyInfo>;
    fn create(&self, input: PtyCreateInput) -> PtyInfo;
    fn get(&self, id: &str) -> Option<PtyInfo>;
    fn update(&self, id: &str, input: PtyUpdateInput) -> Option<PtyInfo>;
    fn connect_token(&self, id: &str) -> Option<PtyTicket>;
    fn attach_stdio(&self, id: &str) -> Option<PtyStdio>;
}

pub struct PtyStdio {
    pub stdin: std::process::ChildStdin,
    pub stdout: std::process::ChildStdout,
}

// ---- Permission Service ----

pub struct PermissionCreateInput {
    pub id: Option<String>,
    pub session_id: String,
    pub action: PermissionAction,
    pub resources: Vec<String>,
    pub save: Option<bool>,
    pub metadata: Option<HashMap<String, String>>,
    pub source: Option<String>,
    pub agent: Option<String>,
}

pub struct PermissionReplyInput {
    pub reply: String,
    pub message: Option<String>,
}

pub trait PermissionService: Send + Sync {
    fn request_list(&self) -> Vec<serde_json::Value>;
    fn saved_list(&self, project_id: Option<String>) -> Vec<serde_json::Value>;
    fn session_list(&self, session_id: &str) -> Vec<serde_json::Value>;
    fn session_create(&self, input: PermissionCreateInput) -> serde_json::Value;
    fn session_get(&self, session_id: &str, request_id: &str) -> Option<serde_json::Value>;
    fn session_reply(&self, session_id: &str, request_id: &str, input: PermissionReplyInput) -> Result<(), ()>;
}

// ---- Question Service ----

pub struct QuestionReplyInput {
    pub answers: Vec<String>,
}

pub trait QuestionService: Send + Sync {
    fn request_list(&self) -> Vec<Question>;
    fn session_list(&self, session_id: &str) -> Vec<Question>;
    fn session_reply(&self, session_id: &str, request_id: &str, input: QuestionReplyInput) -> Result<(), ()>;
    fn session_reject(&self, session_id: &str, request_id: &str) -> Result<(), ()>;
}

// ---- File System Service ----

pub struct FsReadQuery {
    pub path: RelativePath,
}

pub struct FsListQuery {
    pub path: Option<RelativePath>,
}

pub struct FsFindQuery {
    pub query: String,
    pub r#type: String,
    pub limit: Option<u32>,
}

pub struct FsReadResult {
    pub content: Vec<u8>,
    pub mime: String,
}

pub trait FileSystemService: Send + Sync {
    fn read(&self, query: FsReadQuery) -> Result<FsReadResult, String>;
    fn list(&self, query: FsListQuery) -> Vec<serde_json::Value>;
    fn find(&self, query: FsFindQuery) -> Vec<serde_json::Value>;
}

// ---- Integration Service ----

pub struct ConnectKeyInput {
    pub integration_id: String,
    pub key: String,
    pub label: Option<String>,
}

pub struct ConnectOAuthInput {
    pub integration_id: String,
    pub method_id: String,
    pub inputs: HashMap<String, String>,
    pub label: Option<String>,
}

pub trait IntegrationService: Send + Sync {
    fn list(&self) -> Vec<Integration>;
    fn get(&self, id: &str) -> Option<Integration>;
    fn connect_key(&self, input: ConnectKeyInput) -> Result<(), ()>;
    fn connect_oauth(&self, input: ConnectOAuthInput) -> Result<serde_json::Value, ()>;
    fn attempt_status(&self, attempt_id: &str) -> Option<serde_json::Value>;
    fn attempt_complete(&self, attempt_id: &str, code: Option<String>) -> Result<(), String>;
    fn attempt_cancel(&self, attempt_id: &str);
}

// ---- Credential Service ----

pub struct CredentialUpdateInput {
    pub label: String,
}

pub trait CredentialService: Send + Sync {
    fn update(&self, id: &str, input: CredentialUpdateInput) -> Result<(), ()>;
    fn remove(&self, id: &str);
}

// ---- Command Service ----

pub trait CommandService: Send + Sync {
    fn list(&self) -> Vec<Command>;
}

// ---- Skill Service ----

pub trait SkillService: Send + Sync {
    fn list(&self) -> Vec<Skill>;
}

// ---- Reference Service ----

pub trait ReferenceService: Send + Sync {
    fn list(&self) -> Vec<Reference>;
}

// ---- Event Service ----

pub trait EventService: Send + Sync {
    fn subscribe(&self) -> Vec<Event>;
}

// ---- Project Copy Service ----

pub struct ProjectCopyCreateInput {
    pub project_id: String,
    pub source_directory: String,
}

pub trait ProjectCopyService: Send + Sync {
    fn create(&self, input: ProjectCopyCreateInput) -> Result<serde_json::Value, String>;
    fn refresh(&self, project_id: &str) -> Result<(), String>;
}

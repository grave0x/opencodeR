use serde::{Deserialize, Serialize};
use super::agent::AgentID;
use super::location::LocationRef;
use super::model::ModelRef;
use super::project::ProjectID;
use super::revert::RevertState;
use super::schema::{DateTimeUtcFromMillis, RelativePath};
use super::session_id::SessionID;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub id: SessionID,
    pub parent_id: Option<SessionID>,
    pub project_id: ProjectID,
    pub agent: Option<AgentID>,
    pub model: Option<ModelRef>,
    pub cost: f64,
    pub tokens: TokenUsage,
    pub time: SessionTime,
    pub title: String,
    pub location: LocationRef,
    pub subpath: Option<RelativePath>,
    pub revert: Option<RevertState>,
    pub status: SessionStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SessionStatus {
    #[serde(rename = "active")]
    Active,
    #[serde(rename = "paused")]
    Paused,
    #[serde(rename = "frozen")]
    Frozen,
    #[serde(rename = "terminated")]
    Terminated,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TokenUsage {
    pub input: f64,
    pub output: f64,
    pub reasoning: f64,
    pub cache: CacheUsage,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CacheUsage {
    pub read: f64,
    pub write: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionTime {
    pub created: DateTimeUtcFromMillis,
    pub updated: DateTimeUtcFromMillis,
    pub archived: Option<DateTimeUtcFromMillis>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionListAnchor {
    pub id: SessionID,
    pub time: f64,
    pub direction: ListDirection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ListDirection {
    #[serde(rename = "previous")]
    Previous,
    #[serde(rename = "next")]
    Next,
}

// ── Cost breakdown ──────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CostBreakdown {
    pub by_provider: std::collections::HashMap<String, f64>,
    pub by_model: std::collections::HashMap<String, f64>,
    pub total_cost: f64,
    pub total_tokens: TokenUsage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostSummary {
    pub total_sessions: usize,
    pub total_cost: f64,
    pub total_tokens: TokenUsage,
    pub by_provider: std::collections::HashMap<String, f64>,
    pub by_model: std::collections::HashMap<String, f64>,
}

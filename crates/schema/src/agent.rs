use serde::{Deserialize, Serialize};
use super::model::ModelRef;
use super::provider::ProviderRequest;
use super::permission::PermissionRuleset;
use super::schema::PositiveInt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentID(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    pub id: AgentID,
    pub model: Option<ModelRef>,
    pub request: ProviderRequest,
    pub system: Option<String>,
    pub description: Option<String>,
    pub mode: AgentMode,
    pub hidden: bool,
    pub color: Option<AgentColor>,
    pub steps: Option<PositiveInt>,
    pub permissions: PermissionRuleset,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentMode {
    #[serde(rename = "subagent")]
    Subagent,
    #[serde(rename = "primary")]
    Primary,
    #[serde(rename = "all")]
    All,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AgentColor {
    Named(AgentNamedColor),
    Hex(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentNamedColor {
    #[serde(rename = "primary")]
    Primary,
    #[serde(rename = "secondary")]
    Secondary,
    #[serde(rename = "accent")]
    Accent,
    #[serde(rename = "success")]
    Success,
    #[serde(rename = "warning")]
    Warning,
    #[serde(rename = "error")]
    Error,
    #[serde(rename = "info")]
    Info,
}

use serde::{Deserialize, Serialize};

pub type PermissionRuleset = Vec<PermissionRule>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionRule {
    pub action: PermissionAction,
    pub target: PermissionTarget,
    pub allow: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PermissionAction {
    #[serde(rename = "read")]
    Read,
    #[serde(rename = "write")]
    Write,
    #[serde(rename = "execute")]
    Execute,
    #[serde(rename = "admin")]
    Admin,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PermissionTarget {
    #[serde(rename = "filesystem")]
    FileSystem,
    #[serde(rename = "network")]
    Network,
    #[serde(rename = "terminal")]
    Terminal,
    #[serde(rename = "all")]
    All,
    #[serde(untagged)]
    Custom(String),
}

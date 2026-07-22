use opencode_schema::project::ProjectID;
use opencode_schema::schema::{AbsolutePath, NonNegativeInt, PositiveInt, RelativePath};
use opencode_schema::session::ListDirection;
use opencode_schema::workspace::Workspace;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LocationQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub directory: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionsQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace: Option<Workspace>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<PositiveInt>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order: Option<ListDirection>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub directory: Option<AbsolutePath>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project: Option<ProjectID>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subpath: Option<RelativePath>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<SessionsCursor>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionsCursor(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionHistoryQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<PositiveInt>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after: Option<NonNegativeInt>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMessagesQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<NonNegativeInt>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order: Option<ListDirection>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FindQuery {
    pub query: String,
    pub r#type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<PositiveInt>,
    #[serde(flatten)]
    pub location: LocationQuery,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<RelativePath>,
    #[serde(flatten)]
    pub location: LocationQuery,
}

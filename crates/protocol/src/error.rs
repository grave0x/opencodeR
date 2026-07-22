use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvalidRequestError {
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvalidCursorError {
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnauthorizedError {
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForbiddenError {
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionNotFoundError {
    pub session_id: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageNotFoundError {
    pub session_id: String,
    pub message_id: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionNotFoundError {
    pub request_id: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionNotFoundError {
    pub request_id: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderNotFoundError {
    pub provider_id: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PtyNotFoundError {
    pub pty_id: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictError {
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnknownError {
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#ref: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceUnavailableError {
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectCopyError {
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub force_required: Option<bool>,
}

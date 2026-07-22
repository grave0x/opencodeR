use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptInput {
    pub text: String,
    pub sources: Vec<Source>,
    pub attachments: Vec<FileAttachment>,
    pub agent_attachments: Vec<AgentAttachment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Source {
    pub kind: SourceKind,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SourceKind {
    #[serde(rename = "file")]
    File,
    #[serde(rename = "clipboard")]
    Clipboard,
    #[serde(rename = "selection")]
    Selection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileAttachment {
    pub path: String,
    pub content: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentAttachment {
    pub agent_id: String,
    pub context: Option<String>,
}

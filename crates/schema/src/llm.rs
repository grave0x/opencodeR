use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type ProviderMetadata = HashMap<String, HashMap<String, serde_json::Value>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolTextContent {
    #[serde(rename = "type")]
    pub type_: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolFileContent {
    #[serde(rename = "type")]
    pub type_: String,
    pub uri: String,
    pub mime: String,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ToolContent {
    Text(ToolTextContent),
    File(ToolFileContent),
}

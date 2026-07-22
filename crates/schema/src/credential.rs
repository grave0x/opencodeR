use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credential {
    pub id: String,
    pub provider_id: String,
    pub kind: CredentialKind,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CredentialKind {
    #[serde(rename = "api_key")]
    ApiKey,
    #[serde(rename = "oauth")]
    OAuth,
    #[serde(rename = "bearer")]
    Bearer,
}

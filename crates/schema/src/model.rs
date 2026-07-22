use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelRef(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: ModelRef,
    pub provider_id: String,
    pub name: String,
    pub limits: Option<ModelLimits>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelLimits {
    pub max_input: Option<u64>,
    pub max_output: Option<u64>,
}

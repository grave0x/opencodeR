use serde::{Deserialize, Serialize};
use super::schema::DateTimeUtcFromMillis;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: String,
    pub kind: String,
    pub data: serde_json::Value,
    pub timestamp: DateTimeUtcFromMillis,
}

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RelativePath(pub String);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AbsolutePath(pub String);

pub type DateTimeUtcFromMillis = DateTime<Utc>;

pub type PositiveInt = u32;
pub type NonNegativeInt = u32;

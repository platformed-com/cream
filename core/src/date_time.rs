use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DateTime(#[serde(with = "time::serde::rfc3339")] pub OffsetDateTime);

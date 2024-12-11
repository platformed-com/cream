use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

/// Wrapper around `time::OffsetDateTime` which serializes according to RFC3339.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DateTime(#[serde(with = "time::serde::rfc3339")] pub OffsetDateTime);

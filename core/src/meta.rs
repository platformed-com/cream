use serde::Serialize;

use crate::{date_time::DateTime, reference::Reference};

/// Metadata about a resource.
#[derive(Serialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct Meta<R: Serialize> {
    /// The type of resource.
    pub resource_type: R,
    /// When the resource was created.
    pub created: Option<DateTime>,
    /// When the resource was last modified.
    pub last_modified: Option<DateTime>,
    /// The URL of the resource.
    pub location: Option<Reference>,
    /// The current version of the resource.
    pub version: Option<String>,
}

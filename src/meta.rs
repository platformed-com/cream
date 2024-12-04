use serde::Serialize;

use crate::date_time::DateTime;

#[derive(Serialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct Meta<R: Serialize> {
    pub resource_type: R,
    pub created: Option<DateTime>,
    pub last_modified: Option<DateTime>,
    pub location: Option<String>,
    pub version: Option<String>,
}

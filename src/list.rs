use serde::Serialize;

use crate::declare_schema;

declare_schema!(ListResponseSchema = "urn:ietf:params:scim:api:messages:2.0:ListResponse");

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ListResponse<T> {
    pub schemas: [ListResponseSchema; 1],
    pub total_results: usize,
    pub start_index: usize,
    pub items_per_page: usize,
    #[serde(rename = "Resources")]
    pub resources: Vec<T>,
}

impl<T> Default for ListResponse<T> {
    fn default() -> Self {
        Self {
            schemas: Default::default(),
            total_results: Default::default(),
            start_index: 1,
            items_per_page: 100,
            resources: Default::default(),
        }
    }
}

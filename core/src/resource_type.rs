use serde::{Deserialize, Serialize};

use crate::{declare_resource_type, declare_schema, meta::Meta, reference::Reference};

declare_schema!(ResourceTypeSchema = "urn:ietf:params:scim:schemas:core:2.0:ResourceType");
declare_resource_type!(ResourceTypeResourceType = "ResourceType");

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ResourceType {
    #[serde(skip_deserializing)]
    pub schemas: [ResourceTypeSchema; 1],
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub endpoint: String,
    pub schema: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub schema_extensions: Vec<SchemaExtension>,
    #[serde(skip_deserializing)]
    pub meta: Meta<ResourceTypeResourceType>,
}

impl ResourceType {
    pub fn locate(&mut self) {
        self.meta.location = Some(Reference::new_relative(&format!(
            "/ResourceTypes/{}",
            self.name
        )));
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SchemaExtension {
    pub schema: String,
    pub required: bool,
}

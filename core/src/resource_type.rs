use serde::{Deserialize, Serialize};

use crate::{declare_resource_type, declare_schema, meta::Meta, reference::Reference};

declare_schema!(ResourceTypeSchema = "urn:ietf:params:scim:schemas:core:2.0:ResourceType");
declare_resource_type!(ResourceTypeResourceType = "ResourceType");

/// A resource type.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ResourceType {
    /// ["urn:ietf:params:scim:schemas:core:2.0:ResourceType"]
    #[serde(skip_deserializing)]
    pub schemas: [ResourceTypeSchema; 1],
    /// Optional ID for the resource type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Name of this resource type.
    pub name: String,
    /// Description of this resource type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// The path of endpoints relating to this resource type, relative to the SCIM base URL.
    pub endpoint: String,
    /// The core schema ID for this resource type.
    pub schema: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    /// A list of extension schemas for this resource type.
    pub schema_extensions: Vec<SchemaExtension>,
    /// Metadata about this resource type.
    #[serde(skip_deserializing)]
    pub meta: Meta<ResourceTypeResourceType>,
}

impl ResourceType {
    /// Adds the location metadata to this resource type.
    pub fn locate(&mut self) {
        self.meta.location = Some(Reference::new_relative(&format!(
            "/ResourceTypes/{}",
            self.name
        )));
    }
}

/// An extension schema for a resource type.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SchemaExtension {
    /// The schema ID.
    pub schema: String,
    /// Whether attributes from this schema are required when creating a resource of this type.
    pub required: bool,
}

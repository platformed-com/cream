use cream_core::{declare_resource_type, declare_schema, Meta};
use serde::{Deserialize, Serialize};

declare_schema!(
    ServiceProviderConfigSchema = "urn:ietf:params:scim:schemas:core:2.0:ServiceProviderConfig"
);
declare_resource_type!(ServiceProviderConfigResourceType = "ServiceProviderConfig");

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ServiceProviderConfig {
    #[serde(skip_deserializing)]
    pub schemas: [ServiceProviderConfigSchema; 1],
    #[serde(skip_serializing_if = "Option::is_none")]
    pub documentation_uri: Option<String>,
    pub patch: ServiceProviderConfigPatch,
    pub bulk: ServiceProviderConfigBulk,
    pub filter: ServiceProviderConfigFilter,
    pub change_password: ServiceProviderConfigChangePassword,
    pub sort: ServiceProviderConfigSort,
    pub etag: ServiceProviderConfigEtag,
    pub authentication_schemes: Vec<ServiceProviderConfigAuthenticationScheme>,
    #[serde(skip_deserializing)]
    pub meta: Meta<ServiceProviderConfigResourceType>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ServiceProviderConfigPatch {
    pub supported: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ServiceProviderConfigBulk {
    pub supported: bool,
    pub max_operations: i32,
    pub max_payload_size: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ServiceProviderConfigFilter {
    pub supported: bool,
    pub max_results: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ServiceProviderConfigChangePassword {
    pub supported: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ServiceProviderConfigSort {
    pub supported: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ServiceProviderConfigEtag {
    pub supported: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ServiceProviderConfigAuthenticationScheme {
    pub name: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spec_uri: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub documentation_uri: Option<String>,
    #[serde(rename = "type")]
    pub type_: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub primary: Option<bool>,
}

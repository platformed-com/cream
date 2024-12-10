use serde::{Deserialize, Serialize};

use crate::{declare_resource_type, declare_schema, meta::Meta, reference::Reference};

declare_schema!(SchemaSchema = "urn:ietf:params:scim:schemas:core:2.0:Schema");
declare_resource_type!(SchemaResourceType = "Schema");

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Schema {
    #[serde(skip_deserializing)]
    pub schemas: [SchemaSchema; 1],
    pub id: String,
    pub name: String,
    pub description: String,
    pub attributes: Vec<Attribute>,
    #[serde(skip_deserializing)]
    pub meta: Meta<SchemaResourceType>,
}

impl Schema {
    pub fn locate(&mut self) {
        self.meta.location = Some(Reference::new_relative(&format!("/Schemas/{}", self.id)));
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Attribute {
    pub name: String,
    #[serde(rename = "type")]
    pub type_: Type,
    #[serde(default)]
    pub multi_valued: bool,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub required: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub canonical_values: Option<Vec<String>>,
    #[serde(default)]
    pub case_exact: bool,
    #[serde(default)]
    pub mutability: Mutability,
    #[serde(default)]
    pub returned: Returned,
    #[serde(default)]
    pub uniqueness: Uniqueness,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reference_types: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "<[Attribute]>::is_empty")]
    pub sub_attributes: Vec<Attribute>,
}

impl Attribute {
    pub fn new(name: String, type_: Type) -> Self {
        Self {
            name,
            type_,
            multi_valued: false,
            description: "".into(),
            required: false,
            canonical_values: None,
            case_exact: false,
            mutability: Mutability::ReadWrite,
            returned: Returned::Default,
            uniqueness: Uniqueness::None,
            reference_types: None,
            sub_attributes: Vec::new(),
        }
    }
    pub fn multi_valued(mut self) -> Self {
        self.multi_valued = true;
        self
    }
    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }
    pub fn case_exact(mut self) -> Self {
        self.case_exact = true;
        self
    }
    pub fn sub_attributes(mut self, sub_attributes: Vec<Attribute>) -> Self {
        self.sub_attributes = sub_attributes;
        self
    }
    pub fn immutable(mut self) -> Self {
        self.mutability = Mutability::Immutable;
        self
    }
}

#[allow(unused)]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub enum Mutability {
    ReadOnly,
    ReadWrite,
    Immutable,
    WriteOnly,
}

impl Default for Mutability {
    fn default() -> Self {
        Self::ReadWrite
    }
}

#[allow(unused)]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub enum Returned {
    Always,
    Never,
    Default,
    Request,
}

impl Default for Returned {
    fn default() -> Self {
        Self::Default
    }
}

#[allow(unused)]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub enum Uniqueness {
    None,
    Server,
    Global,
}

impl Default for Uniqueness {
    fn default() -> Self {
        Self::None
    }
}

#[allow(unused)]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub enum Type {
    String,
    Boolean,
    Decimal,
    Integer,
    DateTime,
    Binary,
    Reference,
    Complex,
}

impl Default for Type {
    fn default() -> Self {
        Self::String
    }
}

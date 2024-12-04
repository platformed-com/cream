use serde::Serialize;

use crate::{declare_resource_type, declare_schema, meta::Meta};

declare_schema!(SchemaSchema = "urn:ietf:params:scim:schemas:core:2.0:Schema");
declare_resource_type!(SchemaResourceType = "Schema");

#[derive(Serialize, Debug, Clone)]
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
    pub fn locate(&mut self, base_url: &str) {
        self.meta.location = Some(format!("{}/Schemas/{}", base_url, self.id));
    }
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Attribute {
    pub name: String,
    #[serde(rename = "type")]
    pub type_: Type,
    pub multi_valued: bool,
    pub description: String,
    pub required: bool,
    pub canonical_values: Vec<String>,
    pub case_exact: bool,
    pub mutability: Mutability,
    pub returned: Returned,
    pub uniqueness: Uniqueness,
    pub reference_types: Vec<String>,
    #[serde(skip_serializing_if = "<[Attribute]>::is_empty")]
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
            canonical_values: Vec::new(),
            case_exact: false,
            mutability: Mutability::ReadWrite,
            returned: Returned::Default,
            uniqueness: Uniqueness::None,
            reference_types: Vec::new(),
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
#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub enum Mutability {
    ReadOnly,
    ReadWrite,
    Immutable,
    WriteOnly,
}

#[allow(unused)]
#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub enum Returned {
    Always,
    Never,
    Default,
    Request,
}

#[allow(unused)]
#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub enum Uniqueness {
    None,
    Server,
    Global,
}

#[allow(unused)]
#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub enum Type {
    String,
    Boolean,
    Decimal,
    Integer,
    DateTime,
    Reference,
    Complex,
}

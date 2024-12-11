use serde::{Deserialize, Serialize};

use crate::{declare_resource_type, declare_schema, meta::Meta, reference::Reference};

declare_schema!(SchemaSchema = "urn:ietf:params:scim:schemas:core:2.0:Schema");
declare_resource_type!(SchemaResourceType = "Schema");

/// A SCIM schema
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Schema {
    /// ["urn:ietf:params:scim:schemas:core:2.0:Schema"]
    #[serde(skip_deserializing)]
    pub schemas: [SchemaSchema; 1],
    /// The unique identifier for the schema.
    pub id: String,
    /// The name of the schema.
    pub name: String,
    /// A human-readable description of the schema.
    pub description: String,
    /// A list of attributes that form the schema.
    pub attributes: Vec<Attribute>,
    /// Metadata about the schema.
    #[serde(skip_deserializing)]
    pub meta: Meta<SchemaResourceType>,
}

impl Schema {
    /// Adds the location metadata to this schema.
    pub fn locate(&mut self) {
        self.meta.location = Some(Reference::new_relative(&format!("/Schemas/{}", self.id)));
    }
}

/// A single attribute of a SCIM schema.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Attribute {
    /// The name of the attribute.
    pub name: String,
    /// The data type of the attribute.
    #[serde(rename = "type")]
    pub type_: Type,
    /// Whether the attribute is multi-valued.
    #[serde(default)]
    pub multi_valued: bool,
    /// A human-readable description of the attribute.
    #[serde(default)]
    pub description: String,
    /// Whether the attribute is required.
    #[serde(default)]
    pub required: bool,
    /// A list of canonical values for the attribute.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub canonical_values: Option<Vec<String>>,
    /// Whether the attribute is case-sensitive.
    #[serde(default)]
    pub case_exact: bool,
    /// The mutability of the attribute.
    #[serde(default)]
    pub mutability: Mutability,
    /// When the attribute is returned.
    #[serde(default)]
    pub returned: Returned,
    /// The uniqueness of the attribute.
    #[serde(default)]
    pub uniqueness: Uniqueness,
    /// If this attribute is a reference, the types of resources it references.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reference_types: Option<Vec<String>>,
    /// If this attribute is a complex type, the sub-attributes that form it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sub_attributes: Option<Vec<Attribute>>,
}

impl Attribute {
    /// Construct a new attribute.
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
            sub_attributes: None,
        }
    }
    /// Set whether the attribute is multi-valued.
    pub fn multi_valued(mut self) -> Self {
        self.multi_valued = true;
        self
    }
    /// Set whether the attribute is required.
    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }
    /// Set whether the attribute is case-sensitive.
    pub fn case_exact(mut self) -> Self {
        self.case_exact = true;
        self
    }
    /// Set the sub-attributes of this attribute.
    pub fn sub_attributes(mut self, sub_attributes: Vec<Attribute>) -> Self {
        self.sub_attributes = Some(sub_attributes);
        self
    }
    /// Set this attribute as immutable
    pub fn immutable(mut self) -> Self {
        self.mutability = Mutability::Immutable;
        self
    }
    /// Set this attribute as read-only
    pub fn read_only(mut self) -> Self {
        self.mutability = Mutability::ReadOnly;
        self
    }
    /// Set this attribute as write-only
    pub fn write_only(mut self) -> Self {
        self.mutability = Mutability::WriteOnly;
        self
    }
    /// Set this attribute as always returned
    pub fn always_returned(mut self) -> Self {
        self.returned = Returned::Always;
        self
    }
    /// Set this attribute as never returned
    pub fn never_returned(mut self) -> Self {
        self.returned = Returned::Never;
        self
    }
    /// Set this attribute as returned on request
    pub fn returned_on_request(mut self) -> Self {
        self.returned = Returned::Request;
        self
    }
    /// Set this attribute as unique on this server
    pub fn unique(mut self) -> Self {
        self.uniqueness = Uniqueness::Server;
        self
    }
    /// Set this attribute as globally unique
    pub fn globally_unique(mut self) -> Self {
        self.uniqueness = Uniqueness::Global;
        self
    }
    /// Set the types of resources this attribute can reference.
    pub fn reference_types(mut self, reference_types: Vec<String>) -> Self {
        self.reference_types = Some(reference_types);
        self
    }
}

/// The mutability of an attribute.
#[allow(unused)]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub enum Mutability {
    /// Clients can only read this attribute.
    ReadOnly,
    /// Clients can read and write this attribute (default).
    ReadWrite,
    /// Clients can set this value on creation, but not update it.
    Immutable,
    /// Clients can write this attribute, but not read it.
    WriteOnly,
}

impl Default for Mutability {
    fn default() -> Self {
        Self::ReadWrite
    }
}

/// When an attribute is returned.
#[allow(unused)]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub enum Returned {
    /// The attribute is always returned.
    Always,
    /// The attribute is never returned.
    Never,
    /// The attribute is returned by default (default).
    Default,
    /// The attribute is returned on request.
    Request,
}

impl Default for Returned {
    fn default() -> Self {
        Self::Default
    }
}

/// The uniqueness of an attribute.
#[allow(unused)]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub enum Uniqueness {
    /// The attribute is not unique (default).
    None,
    /// The attribute is unique on this server.
    Server,
    /// The attribute is globally unique.
    Global,
}

impl Default for Uniqueness {
    fn default() -> Self {
        Self::None
    }
}

/// The data type of an attribute.
#[allow(unused)]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub enum Type {
    /// A string.
    String,
    /// A boolean.
    Boolean,
    /// A decimal number.
    Decimal,
    /// An integer.
    Integer,
    /// A date and time.
    DateTime,
    /// Binary data (encoded in base64).
    Binary,
    /// A reference to another resource or external URL.
    Reference,
    /// A complex type with sub-attributes.
    Complex,
}

impl Default for Type {
    fn default() -> Self {
        Self::String
    }
}

use serde::{Deserialize, Serialize};

use crate::{
    declare_resource_type, declare_schema,
    filter::{AttrPath, AttrPathRef},
    meta::Meta,
};

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
    pub fn locate(&mut self, base_url: &str) {
        self.meta.location = Some(format!("{}/Schemas/{}", base_url, self.id));
    }
    fn fix_attribute_casing_inner(&self, name: &mut String, parent_name: Option<&str>) {
        if let Some(parent_name) = parent_name {
            if let Some(parent_attr) = self
                .attributes
                .iter()
                .find(|a| a.name.eq_ignore_ascii_case(parent_name))
            {
                if let Some(sub_attr) = parent_attr
                    .sub_attributes
                    .iter()
                    .find(|a| a.name.eq_ignore_ascii_case(name))
                {
                    if sub_attr.name != *name {
                        *name = sub_attr.name.clone();
                    }
                }
            }
        } else if let Some(attr) = self
            .attributes
            .iter()
            .find(|a| a.name.eq_ignore_ascii_case(name))
        {
            if attr.name != *name {
                *name = attr.name.clone();
            }
        }
    }
    pub(crate) fn fix_attribute_casing(
        &self,
        attr: &mut AttrPath,
        parent_name: Option<&str>,
        is_core: bool,
    ) {
        if is_core {
            attr.urn = None;
        } else {
            attr.urn = Some(self.id.clone());
        }
        self.fix_attribute_casing_inner(&mut attr.name, parent_name);
        if let Some(sub_attr) = &mut attr.sub_attr {
            self.fix_attribute_casing_inner(sub_attr, Some(&attr.name));
        }
    }
    pub(crate) fn list_optional_attributes(
        &self,
        include: &[AttrPathRef],
        exclude: &[AttrPathRef],
        is_core: bool,
    ) -> Vec<AttrPathRef> {
        let mut attrs = Vec::new();
        for attr in &self.attributes {
            let path = attr.as_attr(self, is_core, None);
            match attr.returned {
                Returned::Always => {}
                Returned::Never => {}
                Returned::Default => {
                    if !exclude.contains(&path) {
                        attrs.push(path)
                    }
                }
                Returned::Request => {
                    if include.contains(&path) {
                        attrs.push(path)
                    }
                }
            }
            for sub_attr in &attr.sub_attributes {
                let sub_path = sub_attr.as_attr(self, is_core, Some(path));
                match sub_attr.returned {
                    Returned::Always => {}
                    Returned::Never => {}
                    Returned::Default => {
                        if !exclude.contains(&sub_path) {
                            attrs.push(sub_path)
                        }
                    }
                    Returned::Request => {
                        if include.contains(&sub_path) {
                            attrs.push(sub_path)
                        }
                    }
                }
            }
        }
        attrs
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
    #[serde(default)]
    pub canonical_values: Vec<String>,
    #[serde(default)]
    pub case_exact: bool,
    #[serde(default)]
    pub mutability: Mutability,
    #[serde(default)]
    pub returned: Returned,
    #[serde(default)]
    pub uniqueness: Uniqueness,
    #[serde(default)]
    pub reference_types: Vec<String>,
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

    fn as_attr<'a>(
        &'a self,
        schema: &'a Schema,
        is_core: bool,
        parent: Option<AttrPathRef<'a>>,
    ) -> AttrPathRef<'a> {
        if let Some(parent) = parent {
            AttrPathRef {
                urn: parent.urn,
                name: parent.name,
                sub_attr: Some(self.name.as_str()),
            }
        } else {
            AttrPathRef {
                urn: if is_core { None } else { Some(&schema.id) },
                name: self.name.as_str(),
                sub_attr: None,
            }
        }
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

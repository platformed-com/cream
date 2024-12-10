use std::str::FromStr;

use bumpalo::Bump;
use cream_core::{ResourceType, Schema};
use ijson::IValue;
use serde::{Deserialize, Deserializer};

use crate::{
    filter::{self, AttrPath, AttrPathRef, Visitor as _},
    manager::SortOrder,
    schema, Cream, Error,
};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ListResourcesArgs {
    // Filtering
    pub(crate) filter: Option<String>,
    // Sorting
    pub(crate) sort_by: Option<String>,
    #[serde(default)]
    pub(crate) sort_order: SortOrder,
    // Pagination
    pub(crate) start_index: Option<usize>,
    pub(crate) count: Option<usize>,
    // Selection
    #[serde(default, deserialize_with = "deserialize_multistring")]
    pub(crate) attributes: Vec<String>,
    #[serde(default, deserialize_with = "deserialize_multistring")]
    pub(crate) excluded_attributes: Vec<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GetResourcesArgs {
    // Selection
    #[serde(default, deserialize_with = "deserialize_multistring")]
    pub(crate) attributes: Vec<String>,
    #[serde(default, deserialize_with = "deserialize_multistring")]
    pub(crate) excluded_attributes: Vec<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PatchResourceArgs {
    #[serde(rename = "Operations")]
    pub(crate) operations: Vec<PatchOperation>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PatchOperation {
    pub(crate) op: PatchOperationType,
    #[serde(default)]
    pub(crate) path: String,
    #[serde(default)]
    pub(crate) value: IValue,
}

pub(crate) enum PatchOperationType {
    Add,
    Replace,
    Remove,
}

impl FromStr for PatchOperationType {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.eq_ignore_ascii_case("add") {
            Ok(Self::Add)
        } else if s.eq_ignore_ascii_case("replace") {
            Ok(Self::Replace)
        } else if s.eq_ignore_ascii_case("remove") {
            Ok(Self::Remove)
        } else {
            Err(Error::expected("Patch Operation"))
        }
    }
}

serde_plain::derive_deserialize_from_fromstr!(PatchOperationType, "Patch Operation");

#[derive(Deserialize)]
#[serde(untagged)]
enum MultiString {
    CommaSeparated(String),
    Array(Vec<String>),
}

fn deserialize_multistring<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    MultiString::deserialize(deserializer).map(|ms| match ms {
        MultiString::CommaSeparated(s) => s.split(',').map(str::to_string).collect(),
        MultiString::Array(a) => a,
    })
}

pub(crate) struct FixAttributeCasingVisitor<'a> {
    pub(crate) schema: &'a Schema,
    pub(crate) extension_schemas: Vec<&'a Schema>,
    pub(crate) parent_attr: Option<AttrPath>,
}
impl<'a> FixAttributeCasingVisitor<'a> {
    pub(crate) fn new(resource_type: &'a ResourceType, state: &'a Cream) -> Self {
        Self {
            schema: state
                .0
                .schemas
                .get(&resource_type.schema)
                .expect("Resource type references non-existent schema"),
            extension_schemas: resource_type
                .schema_extensions
                .iter()
                .map(|ext| {
                    state
                        .0
                        .schemas
                        .get(&ext.schema)
                        .expect("Resource type references non-existent schema extension")
                })
                .collect(),
            parent_attr: None,
        }
    }
}

impl filter::Visitor for FixAttributeCasingVisitor<'_> {
    fn visit_filter(&mut self, filter: &mut filter::Filter) {
        match filter {
            filter::Filter::Has(parent, _) => {
                self.parent_attr = Some(parent.clone());
                filter::default_visit_filter(self, filter);
                self.parent_attr = None;
            }
            _ => filter::default_visit_filter(self, filter),
        }
    }
    fn visit_attr_path(&mut self, attr_path: &mut AttrPath) {
        if let Some(attr_urn) = attr_path
            .urn
            .as_ref()
            .or(self.parent_attr.as_ref().and_then(|a| a.urn.as_ref()))
        {
            if attr_urn.eq_ignore_ascii_case(&self.schema.id) {
                schema::fix_attribute_casing(
                    self.schema,
                    attr_path,
                    self.parent_attr.as_ref().map(|a| a.name.as_str()),
                    true,
                );
            } else {
                for extension_schema in &self.extension_schemas {
                    if attr_urn.eq_ignore_ascii_case(&extension_schema.id) {
                        schema::fix_attribute_casing(
                            extension_schema,
                            attr_path,
                            self.parent_attr.as_ref().map(|a| a.name.as_str()),
                            false,
                        );
                        break;
                    }
                }
            }
        } else {
            schema::fix_attribute_casing(
                self.schema,
                attr_path,
                self.parent_attr.as_ref().map(|a| a.name.as_str()),
                true,
            );
        }
        filter::default_visit_attr_path(self, attr_path);
    }
}

// Given a list of attribute names as strings, decodes them and fixes the casing to match
// the schema.
fn decode_and_fix_attributes<'a>(
    attributes: &[String],
    fixer: &mut FixAttributeCasingVisitor,
    scope: &'a Bump,
) -> Result<Vec<AttrPathRef<'a>>, Error> {
    let mut decoded = attributes
        .iter()
        .map(|x| filter::parse_attr_path(x))
        .collect::<Result<Vec<_>, _>>()?;
    for item in &mut decoded {
        fixer.visit_attr_path(item);
    }
    let decoded = scope.alloc(decoded);
    Ok(decoded.iter().map(|a| a.as_ref()).collect())
}

pub(crate) fn list_optional_attributes<'a>(
    attributes: &[String],
    excluded_attributes: &[String],
    fixer: &'a mut FixAttributeCasingVisitor,
    scope: &'a Bump,
) -> Result<Vec<AttrPathRef<'a>>, Error> {
    let include = decode_and_fix_attributes(attributes, fixer, scope)?;
    let exclude = decode_and_fix_attributes(excluded_attributes, fixer, scope)?;

    let mut optional_attributes = Vec::new();
    optional_attributes.extend(schema::list_optional_attributes(
        fixer.schema,
        &include,
        &exclude,
        true,
    ));
    for &extension in &fixer.extension_schemas {
        optional_attributes.extend(schema::list_optional_attributes(
            extension, &include, &exclude, false,
        ));
    }
    Ok(optional_attributes)
}

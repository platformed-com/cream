use cream_core::{Attribute, Returned, Schema};

use crate::filter::{AttrPath, AttrPathRef};

fn fix_attribute_casing_inner(schema: &Schema, name: &mut String, parent_name: Option<&str>) {
    if let Some(parent_name) = parent_name {
        if let Some(parent_attr) = schema
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
    } else if let Some(attr) = schema
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
    schema: &Schema,
    attr: &mut AttrPath,
    parent_name: Option<&str>,
    is_core: bool,
) {
    if is_core {
        attr.urn = None;
    } else {
        attr.urn = Some(schema.id.clone());
    }
    fix_attribute_casing_inner(schema, &mut attr.name, parent_name);
    if let Some(sub_attr) = &mut attr.sub_attr {
        fix_attribute_casing_inner(schema, sub_attr, Some(&attr.name));
    }
}
pub(crate) fn list_optional_attributes<'a>(
    schema: &'a Schema,
    include: &[AttrPathRef],
    exclude: &[AttrPathRef],
    is_core: bool,
) -> Vec<AttrPathRef<'a>> {
    let mut attrs = Vec::new();
    for attr in &schema.attributes {
        let path = as_attr(schema, attr, is_core, None);
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
            let sub_path = as_attr(schema, sub_attr, is_core, Some(path));
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

fn as_attr<'a>(
    schema: &'a Schema,
    attribute: &'a Attribute,
    is_core: bool,
    parent: Option<AttrPathRef<'a>>,
) -> AttrPathRef<'a> {
    if let Some(parent) = parent {
        AttrPathRef {
            urn: parent.urn,
            name: parent.name,
            sub_attr: Some(attribute.name.as_str()),
        }
    } else {
        AttrPathRef {
            urn: if is_core { None } else { Some(&schema.id) },
            name: attribute.name.as_str(),
            sub_attr: None,
        }
    }
}

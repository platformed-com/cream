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
                .flatten()
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

// Whether an optional attribute is returned. `included` and `excluded` say whether the
// `attributes` and `excludedAttributes` parameters name it, and `restricted` whether `attributes`
// applies to it at all.
fn is_selected(returned: &Returned, included: bool, excluded: bool, restricted: bool) -> bool {
    let wanted = match returned {
        // RFC 7643 §7: once `attributes` is given, only the attributes it names are returned.
        Returned::Default => included || !restricted,
        Returned::Request => included,
        Returned::Always | Returned::Never => false,
    };
    wanted && !excluded
}

pub(crate) fn list_optional_attributes<'a>(
    schema: &'a Schema,
    include: &[AttrPathRef],
    exclude: &[AttrPathRef],
    is_core: bool,
) -> Vec<AttrPathRef<'a>> {
    let restricted = !include.is_empty();
    let mut attrs = Vec::new();
    for attr in &schema.attributes {
        let path = as_attr(schema, attr, is_core, None);
        let sub_attrs = attr.sub_attributes.as_deref().unwrap_or_default();
        let sub_path = |sub_attr| as_attr(schema, sub_attr, is_core, Some(path));

        match attr.returned {
            Returned::Never => continue,
            Returned::Always => {}
            Returned::Default | Returned::Request => {
                // Naming a sub-attribute returns its parent, containing only that sub-attribute.
                let sub_attr_named = sub_attrs.iter().any(|sub_attr| {
                    !matches!(sub_attr.returned, Returned::Never)
                        && include.contains(&sub_path(sub_attr))
                });
                let included = include.contains(&path) || sub_attr_named;
                let excluded = exclude.contains(&path);
                if !is_selected(&attr.returned, included, excluded, restricted) {
                    continue;
                }
                attrs.push(path);
            }
        }

        // An always-returned attribute is returned "regardless of the contents of the
        // "attributes" parameter", so its sub-attributes keep their defaults.
        let sub_restricted = restricted && !matches!(attr.returned, Returned::Always);
        for sub_attr in sub_attrs {
            let sub_path = sub_path(sub_attr);
            // Naming a complex attribute names all of its sub-attributes.
            let included = include.contains(&sub_path) || include.contains(&path);
            let excluded = exclude.contains(&sub_path) || exclude.contains(&path);
            if is_selected(&sub_attr.returned, included, excluded, sub_restricted) {
                attrs.push(sub_path);
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

#[cfg(test)]
mod tests {
    use cream_core::{Attribute, Schema, Type};

    use super::list_optional_attributes;
    use crate::filter::AttrPathRef;

    const EXT: &str = "urn:example:ext";

    fn path<'a>(name: &'a str, sub_attr: Option<&'a str>) -> AttrPathRef<'a> {
        AttrPathRef {
            urn: None,
            name,
            sub_attr,
        }
    }

    fn string(name: &str) -> Attribute {
        Attribute::new(name.into(), Type::String)
    }

    fn complex(name: &str, sub_attributes: Vec<Attribute>) -> Attribute {
        Attribute::new(name.into(), Type::Complex).sub_attributes(sub_attributes)
    }

    fn schema(id: &str, attributes: Vec<Attribute>) -> Schema {
        Schema {
            schemas: Default::default(),
            id: id.into(),
            name: "Test".into(),
            description: "".into(),
            attributes,
            meta: Default::default(),
        }
    }

    fn user_schema() -> Schema {
        schema(
            "core",
            vec![
                string("userName"),
                string("nickName").returned_on_request(),
                string("password").never_returned(),
                complex(
                    "name",
                    vec![
                        string("givenName"),
                        string("familyName"),
                        string("middleName").returned_on_request(),
                        string("secret").never_returned(),
                    ],
                ),
                complex("requested", vec![string("value")]).returned_on_request(),
                complex("hidden", vec![string("value")]).never_returned(),
            ],
        )
    }

    fn list<'a>(
        schema: &'a Schema,
        include: &[AttrPathRef],
        exclude: &[AttrPathRef],
    ) -> Vec<AttrPathRef<'a>> {
        list_optional_attributes(schema, include, exclude, true)
    }

    #[test]
    fn defaults_without_parameters() {
        let schema = user_schema();
        assert_eq!(
            list(&schema, &[], &[]),
            [
                path("userName", None),
                path("name", None),
                path("name", Some("givenName")),
                path("name", Some("familyName")),
            ]
        );
    }

    #[test]
    fn attributes_replaces_the_defaults() {
        let schema = user_schema();
        assert_eq!(
            list(&schema, &[path("userName", None)], &[]),
            [path("userName", None)]
        );
    }

    #[test]
    fn attributes_selects_request_attributes() {
        let schema = user_schema();
        assert_eq!(
            list(&schema, &[path("nickName", None)], &[]),
            [path("nickName", None)]
        );
        assert_eq!(
            list(&schema, &[path("requested", None)], &[]),
            [path("requested", None), path("requested", Some("value"))]
        );
    }

    #[test]
    fn attributes_never_selects_never_returned_attributes() {
        let schema = user_schema();
        let include = [
            path("password", None),
            path("name", Some("secret")),
            path("hidden", Some("value")),
        ];
        assert_eq!(list(&schema, &include, &[]), []);
    }

    #[test]
    fn naming_a_complex_attribute_names_its_sub_attributes() {
        let schema = user_schema();
        assert_eq!(
            list(&schema, &[path("name", None)], &[]),
            [
                path("name", None),
                path("name", Some("givenName")),
                path("name", Some("familyName")),
                path("name", Some("middleName")),
            ]
        );
    }

    #[test]
    fn naming_a_sub_attribute_returns_only_it_within_its_parent() {
        let schema = user_schema();
        assert_eq!(
            list(&schema, &[path("name", Some("givenName"))], &[]),
            [path("name", None), path("name", Some("givenName"))]
        );
        assert_eq!(
            list(&schema, &[path("requested", Some("value"))], &[]),
            [path("requested", None), path("requested", Some("value"))]
        );
    }

    #[test]
    fn excluding_a_complex_attribute_excludes_its_sub_attributes() {
        let schema = user_schema();
        assert_eq!(
            list(&schema, &[], &[path("name", None)]),
            [path("userName", None)]
        );
        assert_eq!(
            list(
                &schema,
                &[path("name", Some("givenName"))],
                &[path("name", None)]
            ),
            []
        );
    }

    #[test]
    fn excluding_a_sub_attribute_keeps_its_parent() {
        let schema = user_schema();
        assert_eq!(
            list(&schema, &[], &[path("name", Some("givenName"))]),
            [
                path("userName", None),
                path("name", None),
                path("name", Some("familyName")),
            ]
        );
    }

    #[test]
    fn always_returned_attributes_keep_their_default_sub_attributes() {
        let schema = schema(
            "core",
            vec![
                string("userName"),
                complex(
                    "pinned",
                    vec![string("value"), string("extra").returned_on_request()],
                )
                .always_returned(),
            ],
        );
        assert_eq!(
            list(&schema, &[path("userName", None)], &[]),
            [path("userName", None), path("pinned", Some("value"))]
        );
        assert_eq!(
            list(&schema, &[path("pinned", Some("extra"))], &[]),
            [path("pinned", Some("value")), path("pinned", Some("extra"))]
        );
        assert_eq!(
            list(&schema, &[], &[path("pinned", Some("value"))]),
            [path("userName", None)]
        );
    }

    #[test]
    fn extension_attributes_are_matched_with_their_urn() {
        let schema = schema(EXT, vec![string("userName"), string("nickName")]);
        let ext_path = |name| AttrPathRef {
            urn: Some(EXT),
            name,
            sub_attr: None,
        };
        assert_eq!(
            list_optional_attributes(&schema, &[ext_path("nickName")], &[], false),
            [ext_path("nickName")]
        );
        // A core attribute with the same name does not select the extension's attribute.
        assert_eq!(
            list_optional_attributes(&schema, &[path("nickName", None)], &[], false),
            []
        );
    }
}

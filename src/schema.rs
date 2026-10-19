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

// Whether an attribute with `"returned": "default"` or `"request"` is selected by the `attributes`
// (`include`) and `excludedAttributes` (`exclude`) parameters. `paths` holds the attribute's own
// path, followed by its parent's for a sub-attribute, since naming a complex attribute names all
// of its sub-attributes.
//
// RFC 7643 §7: once `attributes` is given, `default` attributes are only returned if it names
// them. `include_all_defaults` keeps them for the sub-attributes of an attribute which is always
// returned "regardless of the contents of the "attributes" parameter".
fn is_selected(
    returned: &Returned,
    paths: &[AttrPathRef],
    include: &[AttrPathRef],
    exclude: &[AttrPathRef],
    include_all_defaults: bool,
) -> bool {
    let named_in = |list: &[AttrPathRef]| paths.iter().any(|path| list.contains(path));
    let included = match returned {
        Returned::Always | Returned::Never => false,
        Returned::Default => include_all_defaults || include.is_empty() || named_in(include),
        Returned::Request => named_in(include),
    };
    included && !named_in(exclude)
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
        let sub_attrs: Vec<_> = attr
            .sub_attributes
            .iter()
            .flatten()
            .map(|sub_attr| {
                (
                    &sub_attr.returned,
                    as_attr(schema, sub_attr, is_core, Some(path)),
                )
            })
            .collect();

        let is_returned = match attr.returned {
            Returned::Never => false,
            Returned::Always => true,
            Returned::Default | Returned::Request => {
                // A complex attribute is also returned when one of its sub-attributes is named,
                // so that `attributes=name.givenName` returns `name` containing only `givenName`.
                let sub_attr_named = sub_attrs.iter().any(|(returned, sub_path)| {
                    !matches!(returned, Returned::Never) && include.contains(sub_path)
                });
                let selected = is_selected(&attr.returned, &[path], include, exclude, false)
                    || (sub_attr_named && !exclude.contains(&path));
                if selected {
                    attrs.push(path);
                }
                selected
            }
        };
        if !is_returned {
            continue;
        }

        let include_all_defaults = matches!(attr.returned, Returned::Always);
        for (returned, sub_path) in sub_attrs {
            if is_selected(
                returned,
                &[sub_path, path],
                include,
                exclude,
                include_all_defaults,
            ) {
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

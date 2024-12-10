use crate::AttrPathRef;

pub const META_RESOURCE_TYPE: AttrPathRef = AttrPathRef {
    urn: None,
    name: "meta",
    sub_attr: Some("resourceType"),
};

pub const META_CREATED: AttrPathRef = AttrPathRef {
    urn: None,
    name: "meta",
    sub_attr: Some("created"),
};

pub const META_LAST_MODIFIED: AttrPathRef = AttrPathRef {
    urn: None,
    name: "meta",
    sub_attr: Some("lastModified"),
};

pub const META_VERSION: AttrPathRef = AttrPathRef {
    urn: None,
    name: "meta",
    sub_attr: Some("version"),
};

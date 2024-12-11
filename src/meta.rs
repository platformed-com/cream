use crate::AttrPathRef;

/// Common `meta.resourceType` attribute path.
pub const META_RESOURCE_TYPE: AttrPathRef = AttrPathRef {
    urn: None,
    name: "meta",
    sub_attr: Some("resourceType"),
};

/// Common `meta.created` attribute path.
pub const META_CREATED: AttrPathRef = AttrPathRef {
    urn: None,
    name: "meta",
    sub_attr: Some("created"),
};

/// Common `meta.lastModified` attribute path.
pub const META_LAST_MODIFIED: AttrPathRef = AttrPathRef {
    urn: None,
    name: "meta",
    sub_attr: Some("lastModified"),
};

/// Common `meta.version` attribute path.
pub const META_VERSION: AttrPathRef = AttrPathRef {
    urn: None,
    name: "meta",
    sub_attr: Some("version"),
};

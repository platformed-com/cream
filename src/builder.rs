use std::{collections::BTreeMap, sync::Arc};

use cream_core::Schema;

use crate::{
    config::ServiceProviderConfig,
    manager::GenericResourceManager,
    state::{Cream, InnerState, ResourceTypeState},
};

/// Builder for constructing a `Cream` instance.
pub struct CreamBuilder {
    base_url: String,
    config: ServiceProviderConfig,
    schemas: BTreeMap<String, Schema>,
    resource_types: BTreeMap<String, ResourceTypeState>,
}

impl CreamBuilder {
    /// Create a new `CreamBuilder` with the given base URL and service provider configuration.
    ///
    /// `base_url` is the public URL at which the router is served, without a trailing slash. It is
    /// prepended to relative references such as `meta.location`.
    ///
    /// `config` is returned from `/ServiceProviderConfig`, and is usually loaded from a JSON file
    /// with [`load_static_json!`](crate::load_static_json).
    pub fn new(base_url: &str, config: ServiceProviderConfig) -> Self {
        Self {
            base_url: base_url.to_string(),
            config,
            schemas: BTreeMap::new(),
            resource_types: BTreeMap::new(),
        }
    }
    fn load_schema(&mut self, id: &str, manager: &impl GenericResourceManager) {
        if !self.schemas.contains_key(id) {
            self.schemas.insert(id.into(), manager.load_schema(id));
        }
    }
    /// Add a new resource type to be handled by cream, typically `<Name>::manage(manager)` for a
    /// type declared with [`declare_resource!`](crate::declare_resource).
    pub fn resource_type(mut self, manager: impl GenericResourceManager) -> Self {
        let resource_type = manager.load_resource_type();

        self.load_schema(&resource_type.schema, &manager);
        for ext in &resource_type.schema_extensions {
            self.load_schema(&ext.schema, &manager);
        }

        self.resource_types.insert(
            resource_type.name.clone(),
            ResourceTypeState {
                resource_type,
                manager: Box::new(manager),
            },
        );
        self
    }

    /// Build the `Cream` instance.
    pub fn build(self) -> Cream {
        Cream(Arc::new(InnerState {
            base_url: self.base_url,
            config: self.config,
            schemas: self.schemas,
            resource_types: self.resource_types,
        }))
    }
}

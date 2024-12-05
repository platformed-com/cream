use std::{collections::BTreeMap, sync::Arc};

use crate::{
    config::ServiceProviderConfig,
    manager::ResourceTypeManager,
    resource_type::ResourceType,
    schema::Schema,
    state::{Cream, InnerState, ResourceTypeState},
};

pub struct CreamBuilder {
    base_url: String,
    config: ServiceProviderConfig,
    schemas: BTreeMap<String, Schema>,
    resource_types: BTreeMap<String, ResourceTypeState>,
}

impl CreamBuilder {
    pub fn new(base_url: &str, config: ServiceProviderConfig) -> Self {
        Self {
            base_url: base_url.to_string(),
            config,
            schemas: BTreeMap::new(),
            resource_types: BTreeMap::new(),
        }
    }
    pub fn schema(mut self, schema: Schema) -> Self {
        self.schemas.insert(schema.id.clone(), schema);
        self
    }
    pub fn resource_type(
        mut self,
        resource_type: ResourceType,
        manager: impl ResourceTypeManager,
    ) -> Self {
        self.resource_types.insert(
            resource_type.name.clone(),
            ResourceTypeState {
                resource_type,
                manager: Box::new(manager),
            },
        );
        self
    }
    pub fn build(self) -> Cream {
        Cream(Arc::new(InnerState {
            base_url: self.base_url,
            config: self.config,
            schemas: self.schemas,
            resource_types: self.resource_types,
        }))
    }
}

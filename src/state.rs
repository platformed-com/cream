use std::{collections::BTreeMap, sync::Arc};

use cream_core::{ResourceType, Schema};

use crate::{config::ServiceProviderConfig, manager::GenericResourceManager};

#[derive(Clone, Debug)]
pub struct Cream(pub(crate) Arc<InnerState>);

#[derive(Debug)]
pub(crate) struct ResourceTypeState {
    pub(crate) resource_type: ResourceType,
    pub(crate) manager: Box<dyn GenericResourceManager>,
}

#[derive(Debug)]
pub(crate) struct InnerState {
    pub(crate) base_url: String,
    pub(crate) config: ServiceProviderConfig,
    pub(crate) schemas: BTreeMap<String, Schema>,
    pub(crate) resource_types: BTreeMap<String, ResourceTypeState>,
}

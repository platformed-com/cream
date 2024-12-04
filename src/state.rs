use std::{collections::BTreeMap, sync::Arc};

use crate::{config::ServiceProviderConfig, resource_type::ResourceType, schema::Schema};

#[derive(derive_more::Deref, Clone, Debug)]
pub struct CreamState(Arc<InnerState>);

#[derive(Debug)]
pub struct InnerState {
    pub base_url: String,
    pub config: ServiceProviderConfig,
    pub schemas: BTreeMap<String, Schema>,
    pub resource_types: BTreeMap<String, ResourceType>,
}

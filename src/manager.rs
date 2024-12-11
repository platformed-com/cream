use std::fmt::Debug;

use axum::http::request::Parts;
use cream_core::{ResourceType, Schema};
use ijson::{IObject, IValue};
use serde::{Deserialize, Serialize};

use crate::{
    error::Error,
    filter::{AttrPathRef, FilterRef, ValuePathRef},
};

/// A trait for managing a generic resource. Implemented automatically by the `define_resource` macro.
#[axum::async_trait]
pub trait GenericResourceManager: Debug + Send + Sync + 'static {
    /// List resources.
    async fn list(
        &self,
        parts: &'async_trait Parts,
        args: ListResourceArgs<'async_trait>,
    ) -> Result<ListResourceResult<IObject>, Error>;
    /// Get a resource by ID.
    async fn get(
        &self,
        parts: &'async_trait Parts,
        args: GetResourceArgs<'async_trait>,
    ) -> Result<IObject, Error>;
    /// Create a new resource.
    async fn create(&self, parts: &'async_trait Parts, resource: IObject) -> Result<String, Error>;
    /// Update a resource.
    async fn update(
        &self,
        parts: &'async_trait Parts,
        args: UpdateResourceArgs<'async_trait>,
    ) -> Result<(), Error>;
    /// Replace a resource.
    async fn replace(
        &self,
        parts: &'async_trait Parts,
        id: &str,
        resource: IObject,
    ) -> Result<(), Error>;
    /// Delete a resource by ID.
    async fn delete(&self, parts: &'async_trait Parts, id: &str) -> Result<(), Error>;
    /// Get the default page size for this resource type.
    fn default_page_size(&self) -> usize {
        50
    }

    // Reflection
    /// Load the resource type for this manager.
    fn load_resource_type(&self) -> ResourceType;
    /// Load the schema with the given ID.
    fn load_schema(&self, id: &str) -> Schema;
}

/// Arguments for listing resources.
#[non_exhaustive]
#[derive(Debug, Default)]
pub struct ListResourceArgs<'a> {
    /// Filter to apply to the resources.
    pub filter: Option<FilterRef<'a>>,
    /// Attribute to sort by.
    pub sort_by: Option<AttrPathRef<'a>>,
    /// Sort order.
    pub sort_order: SortOrder,
    /// Index of the first resource to return (zero-indexed).
    pub start_index: usize,
    /// Number of resources to return.
    pub count: usize,
    /// Additional attributes to include in the response.
    pub optional_attributes: &'a [AttrPathRef<'a>],
}

/// Arguments for getting a resource by ID.
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct GetResourceArgs<'a> {
    /// ID of the resource to get.
    pub id: String,
    /// Additional attributes to include in the response.
    pub optional_attributes: &'a [AttrPathRef<'a>],
}

/// Result of listing resources.
#[derive(Debug, Clone, Default)]
pub struct ListResourceResult<T> {
    /// The resources.
    pub resources: Vec<T>,
    /// The page size used for this result.
    pub items_per_page: usize,
    /// The total number of items matching the filter.
    pub total_count: usize,
}

/// Arguments for updating a resource.
#[non_exhaustive]
#[derive(Debug)]
pub struct UpdateResourceArgs<'a> {
    /// ID of the resource to update.
    pub id: &'a str,
    /// The updates to apply.
    pub items: &'a [UpdateResourceItem<'a>],
}

/// An update to apply to a resource.
#[non_exhaustive]
#[derive(Debug, Clone, Copy)]
pub struct UpdateResourceItem<'a> {
    /// Path to the attribute to update.
    pub path: Option<ValuePathRef<'a>>,
    /// The operation to apply.
    pub op: UpdateOp<'a>,
}

/// The type of update to apply to an attribute.
#[derive(Debug, Clone, Copy)]
pub enum UpdateOp<'a> {
    /// Add a value.
    Add(&'a IValue),
    /// Remove a value.
    Remove,
    /// Replace a value in an attribute.
    Replace(&'a IValue),
}

/// Sort order for listing resources.
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub enum SortOrder {
    /// Sort in ascending order (default).
    Ascending,
    /// Sort in descending order.
    Descending,
}

impl Default for SortOrder {
    fn default() -> Self {
        Self::Ascending
    }
}

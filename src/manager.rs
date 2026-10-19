use std::fmt::Debug;

use axum::http::request::Parts;
use cream_core::{ResourceType, Schema};
use ijson::{IObject, IValue};
use serde::{Deserialize, Serialize};

use crate::{
    error::Error,
    filter::{AttrPathRef, FilterRef, ValuePathRef},
};

/// A trait for managing a generic resource.
///
/// Implemented automatically for the managers of types declared with the [`declare_resource!`] macro:
/// `<Name>::manage(manager)` wraps an implementation of the generated `<Name>Manager` trait.
///
/// [`declare_resource!`]: crate::declare_resource
#[async_trait::async_trait]
pub trait GenericResourceManager: Debug + Send + Sync + 'static {
    /// List the resources matching the filter, sorted and paginated as requested.
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
    /// Create a new resource, returning its ID. The response is built by passing that ID to `get`.
    async fn create(&self, parts: &'async_trait Parts, resource: IObject) -> Result<String, Error>;
    /// Apply a PATCH request to a resource. The response is built by calling `get` afterwards.
    async fn update(
        &self,
        parts: &'async_trait Parts,
        args: UpdateResourceArgs<'async_trait>,
    ) -> Result<(), Error>;
    /// Replace a resource in response to a PUT request. The response is built by calling `get` afterwards.
    async fn replace(
        &self,
        parts: &'async_trait Parts,
        id: &str,
        resource: IObject,
    ) -> Result<(), Error>;
    /// Delete a resource by ID.
    async fn delete(&self, parts: &'async_trait Parts, id: &str) -> Result<(), Error>;
    /// The page size used for this resource type when the client does not specify `count`.
    fn default_page_size(&self) -> usize {
        50
    }

    // Reflection
    /// Load the resource type for this manager.
    fn load_resource_type(&self) -> ResourceType;
    /// Load the schema with the given ID. Called for the resource type's core schema and each of its extensions.
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
    /// Maximum number of resources to return: the client's `count`, or
    /// [`GenericResourceManager::default_page_size`] if it did not specify one.
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
    /// The page size used for this result, returned to the client as `itemsPerPage`.
    pub items_per_page: usize,
    /// The total number of items matching the filter across all pages, returned to the client as `totalResults`.
    pub total_count: usize,
}

/// Arguments for updating a resource.
#[non_exhaustive]
#[derive(Debug)]
pub struct UpdateResourceArgs<'a> {
    /// ID of the resource to update.
    pub id: &'a str,
    /// The updates to apply, in order.
    ///
    /// An `add` or `replace` operation without a path, whose value is an object, is split into one
    /// [`UpdateOp::Replace`] per attribute in that object.
    pub items: &'a [UpdateResourceItem<'a>],
}

/// An update to apply to a resource.
#[non_exhaustive]
#[derive(Debug, Clone, Copy)]
pub struct UpdateResourceItem<'a> {
    /// Path to the attribute to update, or `None` if the operation had no path.
    pub path: Option<ValuePathRef<'a>>,
    /// The operation to apply.
    pub op: UpdateOp<'a>,
}

/// The type of update to apply to an attribute.
#[derive(Debug, Clone, Copy)]
pub enum UpdateOp<'a> {
    /// Add a value.
    Add(&'a IValue),
    /// Remove a value. The value is null unless the client sent one, which is a non-standard
    /// extension used by Microsoft Entra. Normally the value to remove is selected by a filter
    /// in the path.
    Remove(&'a IValue),
    /// Replace a value in an attribute.
    Replace(&'a IValue),
}

/// Sort order for listing resources.
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
#[serde(rename_all = "camelCase")]
pub enum SortOrder {
    /// Sort in ascending order (default).
    #[default]
    Ascending,
    /// Sort in descending order.
    Descending,
}

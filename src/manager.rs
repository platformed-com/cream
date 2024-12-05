use std::fmt::Debug;

use ijson::{IObject, IValue};
use serde::{Deserialize, Serialize};

use crate::{
    error::Error,
    filter::{AttrPathRef, FilterRef, ValuePathRef},
};

#[axum::async_trait]
pub trait ResourceTypeManager: Debug + Send + Sync + 'static {
    async fn list(
        &self,
        args: ListResourceArgs<'async_trait>,
    ) -> Result<ListResourceResult<IObject>, Error>;
    async fn get(
        &self,
        args: GetResourceArgs<'async_trait>,
    ) -> Result<ListResourceResult<IObject>, Error>;
    async fn create(&self, resource: IObject) -> Result<IObject, Error>;
    async fn update(&self, args: UpdateResourceArgs<'async_trait>) -> Result<(), Error>;
    fn default_page_size(&self) -> usize {
        50
    }
}

#[derive(Debug, Default)]
pub struct ListResourceArgs<'a> {
    pub filter: Option<FilterRef<'a>>,
    pub sort_by: Option<AttrPathRef<'a>>,
    pub sort_order: SortOrder,
    pub start_index: usize,
    pub count: usize,
    pub optional_attributes: &'a [AttrPathRef<'a>],
}

#[derive(Debug, Clone)]
pub struct GetResourceArgs<'a> {
    pub id: String,
    pub optional_attributes: &'a [AttrPathRef<'a>],
}

#[derive(Debug, Clone, Default)]
pub struct ListResourceResult<T> {
    pub resources: Vec<T>,
    pub items_per_page: usize,
    pub total_count: usize,
}

#[derive(Debug)]
pub struct UpdateResourceArgs<'a> {
    pub id: String,
    pub path: ValuePathRef<'a>,
    pub op: UpdateOp<'a>,
}

#[derive(Debug)]
pub enum UpdateOp<'a> {
    Add(&'a IValue),
    Remove,
    Replace(&'a IValue),
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub enum SortOrder {
    Ascending,
    Descending,
}

impl Default for SortOrder {
    fn default() -> Self {
        Self::Ascending
    }
}

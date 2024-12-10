mod builder;
mod config;
mod error;
pub mod filter;
mod json;
mod list;
mod manager;
mod meta;
mod router;
mod schema;
mod state;

pub use builder::CreamBuilder;
pub use cream_core::*;
pub use cream_macros::*;
pub use error::{Error, ErrorType};
pub use filter::AttrPathRef;
pub use manager::{
    GenericResourceManager, GetResourceArgs, ListResourceArgs, ListResourceResult, UpdateOp,
    UpdateResourceArgs,
};
pub use meta::{META_CREATED, META_LAST_MODIFIED, META_RESOURCE_TYPE, META_VERSION};
pub use state::Cream;

#[doc(hidden)]
pub mod hidden {
    pub use axum;
    pub use ijson;
    pub use serde;
}

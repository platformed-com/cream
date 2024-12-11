//! cream
//!
//! An implementation of SCIM (System for Cross-domain Identity Management).
//!
//! # SCIM Overview
//!
//! SCIM is a standard for automating the exchange of user identity information between identity domains.
//!
//! When an organization purchases a SaaS product, it needs a way to provision, de-provision, and generally manage user accounts
//! within the new product. SCIM allows this management to occur via their existing identity provider, by allowing the identity
//! provider to push user-management changes directly to the SaaS product.
//!
//! SCIM takes the form of a REST API exposed by the service provider, which the identity provider (client) can use to manage users
//! and groups within the SaaS product.
//!
//! # Cream
//!
//! Cream is a Rust implementation of SCIM, designed to be easy to use and flexible.
//!
//! Users of cream define their supported resource types via standard SCIM schemas. Cream then generates Rust code for these types,
//! and exposes an `axum::Router` which can be mounted directly into any `axum` or `tower-http`-based application.
//!
//! SCIM is a complex and underspecified standard, and Cream aims to hide some of this complexity from the user:
//!
//! - Many parts of SCIM are case-insensitive, but some are case-sensitive. Cream uses your schema to normalize the casing on
//!   attributes, schema IDs and filters, so that your application can expect a consistent casing.
//!
//! - SCIM provides many ways to do the same thing. For example, you can search for resources of a particular type via a `GET`
//!   request with query parameters, via a `POST` request with a filter in the body, or by a `POST` to the SCIM base URL with a
//!   filter on the core "resourceType" attribute. Cream ensures you only have to implement a single search method.
//!
//! - SCIM filters are complicated to parse, and may be arbitrarily complicated. Cream handles the parsing and translates them
//!   into Rust-native types which can be directly pattern-matched. This allows you to abstract away subtle differences in the
//!   way different SCIM clients may filter for resources.
//!
//! - SCIM clients can request that some fields be excluded whilst other fields are included. Cream hides this complexity by
//!   giving you a single list of "optional" fields that are to be included along with the required fields which are always
//!   present.
//!
//! Cream supports all aspects of the SCIM v2 standard, with the exception of these optional endpoints:
//! - `/Me`
//!    
//!   This endpoint only makes sense when the SCIM client authenticates as a specific user, which is not part of the typical
//!   SCIM use-case.
//!
//! - `/Bulk`
//!
//!   This endpoint is not yet implemented, but may be added in future.
//!
#![deny(missing_docs)]

mod builder;
mod config;
mod error;
/// Functionality relating to SCIM filters.
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
    GenericResourceManager, GetResourceArgs, ListResourceArgs, ListResourceResult, SortOrder,
    UpdateOp, UpdateResourceArgs, UpdateResourceItem,
};
pub use meta::{META_CREATED, META_LAST_MODIFIED, META_RESOURCE_TYPE, META_VERSION};
pub use state::Cream;

#[doc(hidden)]
pub mod hidden {
    pub use axum;
    pub use ijson;
    pub use serde;
    pub use serde_json;
}

mod date_time;
mod macros;
mod meta;
mod reference;
mod resource_type;
mod schema;

pub use date_time::DateTime;
pub use meta::Meta;
pub use reference::Reference;
pub use resource_type::{ResourceType, SchemaExtension};
pub use schema::{Attribute, Mutability, Returned, Schema, Type, Uniqueness};

#[doc(hidden)]
pub mod hidden {
    #[cfg(feature = "tokio")]
    pub use crate::reference::BASE_URL;
}

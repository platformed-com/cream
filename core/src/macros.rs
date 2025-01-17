/// Declare a schema with a given name and id.
#[macro_export]
macro_rules! declare_schema {
    ($name:ident = $id:literal) => {
        #[derive(derive_more::Display, Default, Debug, Copy, Clone)]
        #[display($id)]
        pub struct $name;

        serde_plain::derive_serialize_from_display!($name);
    };
}

/// Declare a resource type with a given name and id.
#[macro_export]
macro_rules! declare_resource_type {
    ($name:ident = $id:literal) => {
        #[derive(derive_more::Display, Default, Debug, Copy, Clone)]
        #[display($id)]
        pub struct $name;

        serde_plain::derive_serialize_from_display!($name);
    };
}

/// Load a JSON file from the `include_str!` macro.
#[macro_export]
macro_rules! load_static_json {
    ($path:literal) => {
        ::serde_json::from_str(include_str!($path)).expect(concat!("Failed to deserialize ", $path))
    };
}

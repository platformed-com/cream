/// Declare a unit struct `$name` which serializes as the schema URN `$id`.
#[macro_export]
macro_rules! declare_schema {
    ($name:ident = $id:literal) => {
        #[derive(derive_more::Display, Default, Debug, Copy, Clone)]
        #[display($id)]
        pub struct $name;

        serde_plain::derive_serialize_from_display!($name);
    };
}

/// Declare a unit struct `$name` which serializes as the resource type name `$id`.
#[macro_export]
macro_rules! declare_resource_type {
    ($name:ident = $id:literal) => {
        #[derive(derive_more::Display, Default, Debug, Copy, Clone)]
        #[display($id)]
        pub struct $name;

        serde_plain::derive_serialize_from_display!($name);
    };
}

/// Deserialize a JSON file embedded with `include_str!`, so `$path` is relative to the file
/// invoking the macro.
///
/// The calling crate must depend on `serde_json`. Panics if the file does not deserialize.
#[macro_export]
macro_rules! load_static_json {
    ($path:literal) => {
        ::serde_json::from_str(include_str!($path)).expect(concat!("Failed to deserialize ", $path))
    };
}

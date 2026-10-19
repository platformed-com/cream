/// Declare a unit struct `$name` which serializes as the schema URN `$id`.
#[macro_export]
macro_rules! declare_schema {
    ($name:ident = $id:literal) => {
        $crate::__declare_unit_str!($name = $id);
    };
}

/// Declare a unit struct `$name` which serializes as the resource type name `$id`.
#[macro_export]
macro_rules! declare_resource_type {
    ($name:ident = $id:literal) => {
        $crate::__declare_unit_str!($name = $id);
    };
}

// Expands only to paths through `$crate`, so that calling crates need no dependencies besides
// `cream` or `cream-core`.
#[doc(hidden)]
#[macro_export]
macro_rules! __declare_unit_str {
    ($name:ident = $id:literal) => {
        #[derive(Default, Debug, Copy, Clone)]
        pub struct $name;

        impl ::core::fmt::Display for $name {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.write_str($id)
            }
        }

        impl $crate::hidden::serde::Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> ::core::result::Result<S::Ok, S::Error>
            where
                S: $crate::hidden::serde::Serializer,
            {
                serializer.serialize_str($id)
            }
        }
    };
}

/// Deserialize a JSON file embedded with `include_str!`, so `$path` is relative to the file
/// invoking the macro. Panics if the file does not deserialize.
#[macro_export]
macro_rules! load_static_json {
    ($path:literal) => {
        $crate::hidden::serde_json::from_str(include_str!($path))
            .expect(concat!("Failed to deserialize ", $path))
    };
}

# cream

[![crates.io](https://img.shields.io/crates/v/cream.svg)](https://crates.io/crates/cream)
[![docs.rs](https://docs.rs/cream/badge.svg)](https://docs.rs/cream)

An implementation of [SCIM 2.0](https://scim.cloud/) (System for Cross-domain Identity Management) for
[axum](https://github.com/tokio-rs/axum).

SCIM lets an identity provider such as Microsoft Entra ID or Okta provision and de-provision users and groups in your
application. Cream implements the protocol side of [RFC 7644](https://datatracker.ietf.org/doc/html/rfc7644): you describe
your resource types with standard [RFC 7643](https://datatracker.ietf.org/doc/html/rfc7643) schema files, implement a
manager trait for each one, and mount the generated `axum::Router`.

Cream takes care of the parts of SCIM that are easy to get wrong:

- **One search method.** `GET /Users?filter=…`, `POST /Users/.search` and `POST /.search` with a `meta.resourceType`
  filter all reach the same `list` method.
- **Parsed filters.** Filters arrive as Rust enums you can pattern-match against generated attribute constants, such as
  `Compare(User::USER_NAME, Equal, Str(name))`.
- **Normalized casing.** Attribute names and schema URNs in filters, `sortBy`, `attributes`, `excludedAttributes` and
  PATCH paths are rewritten to the casing in your schema.
- **Attribute selection.** `attributes` and `excludedAttributes` are combined with each attribute's `returned`
  characteristic into a single list of optional attributes to include.
- **PATCH normalization.** An `add` or `replace` without a path is split into one update per attribute.
- **Discovery endpoints.** `/ServiceProviderConfig`, `/Schemas` and `/ResourceTypes` are served from your schema files.

## Getting started

Add cream, `async-trait` for implementing the generated manager traits, and axum and tokio to serve the router:

```toml
[dependencies]
cream = "0.2"
async-trait = "0.1"
axum = "0.8"
tokio = { version = "1", features = ["macros", "rt-multi-thread", "net"] }
```

Describe each resource type with JSON files. The standard schemas from RFC 7643 are in [`static/scim`](static/scim)
to copy from, and [`static/jsonschema`](static/jsonschema) has JSON Schemas for editor validation:

```
scim/
├── group.json                    # copy of static/scim/core/group.json
├── group_type.json               # the resource type, as below
└── service_provider_config.json  # served from /ServiceProviderConfig
```

```json
{
  "schemas": ["urn:ietf:params:scim:schemas:core:2.0:ResourceType"],
  "id": "Group",
  "name": "Group",
  "endpoint": "/Groups",
  "description": "Group",
  "schema": "urn:ietf:params:scim:schemas:core:2.0:Group",
  "schemaExtensions": []
}
```

In `service_provider_config.json`, leave `bulk` and `etag` unsupported, since Cream does not implement them. Whether
`filter` and `sort` are supported depends on what your managers handle:

```json
{
  "patch": { "supported": true },
  "bulk": { "supported": false, "maxOperations": 0, "maxPayloadSize": 0 },
  "filter": { "supported": true, "maxResults": 100 },
  "changePassword": { "supported": false },
  "sort": { "supported": false },
  "etag": { "supported": false },
  "authenticationSchemes": [
    {
      "type": "oauthbearertoken",
      "name": "OAuth Bearer Token",
      "description": "Authentication using a bearer token"
    }
  ]
}
```

Then declare the resource, implement its manager, and serve the router:

```rust
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

use axum::http::request::Parts;
use cream::{
    load_static_json, CreamBuilder, Error, GetResourceArgs, ListResourceArgs, ListResourceResult,
    UpdateResourceArgs,
};

mod scim {
    // Paths are relative to the crate root.
    cream::declare_resource!("scim/group_type.json" as Group ["scim/group.json"]);
}

#[derive(Debug, Default, Clone)]
struct Groups(Arc<Mutex<BTreeMap<String, scim::Group>>>);

#[async_trait::async_trait]
impl scim::GroupManager for Groups {
    async fn list(
        &self,
        _parts: &'async_trait Parts,
        args: ListResourceArgs<'async_trait>,
    ) -> Result<ListResourceResult<scim::Group>, Error> {
        use cream::filter::prelude::*;

        // Match the filters you support, and reject the rest.
        let mut display_name = None;
        if let Some(filter) = args.filter {
            for expr in filter.iter_cnf() {
                match expr {
                    Compare(scim::Group::DISPLAY_NAME, Equal, Str(name)) => {
                        display_name = Some(name)
                    }
                    _ => return Err(Error::invalid_filter()),
                }
            }
        }

        let groups = self.0.lock().unwrap();
        let matching: Vec<_> = groups
            .values()
            .filter(|g| display_name.is_none() || g.display_name.as_deref() == display_name)
            .collect();
        Ok(ListResourceResult {
            total_count: matching.len(),
            items_per_page: args.count,
            // `start_index` is zero-based.
            resources: matching
                .into_iter()
                .skip(args.start_index)
                .take(args.count)
                .cloned()
                .collect(),
        })
    }

    async fn get(
        &self,
        _parts: &'async_trait Parts,
        args: GetResourceArgs<'async_trait>,
    ) -> Result<scim::Group, Error> {
        let groups = self.0.lock().unwrap();
        groups.get(&args.id).cloned().ok_or_else(Error::not_found)
    }

    async fn create(
        &self,
        _parts: &'async_trait Parts,
        resource: scim::CreateGroup,
    ) -> Result<String, Error> {
        let mut groups = self.0.lock().unwrap();
        let id = (groups.len() + 1).to_string();
        let group = scim::Group {
            id: id.clone(),
            external_id: resource.external_id,
            display_name: resource.display_name,
            members: None,
            schemas: Default::default(),
            meta: Default::default(),
        };
        groups.insert(id.clone(), group);
        Ok(id)
    }

    // See examples/smoke.rs for implementations of these.
    async fn update(
        &self,
        _parts: &'async_trait Parts,
        _args: UpdateResourceArgs<'async_trait>,
    ) -> Result<(), Error> {
        todo!()
    }

    async fn replace(
        &self,
        _parts: &'async_trait Parts,
        _id: &'async_trait str,
        _resource: scim::CreateGroup,
    ) -> Result<(), Error> {
        todo!()
    }

    async fn delete(
        &self,
        _parts: &'async_trait Parts,
        _id: &'async_trait str,
    ) -> Result<(), Error> {
        todo!()
    }
}

#[tokio::main]
async fn main() {
    let cream = CreamBuilder::new(
        "http://localhost:3000/scim/v2",
        // Paths are relative to this file.
        load_static_json!("../scim/service_provider_config.json"),
    )
    .resource_type(scim::Group::manage(Groups::default()))
    .build();

    let app = axum::Router::new().nest("/scim/v2", cream.router());
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
```

`declare_resource!` generates the `Group` struct returned to clients, the `CreateGroup` struct received in POST and PUT
requests, the `GroupManager` trait, and an associated constant for each attribute. Its paths are relative to the crate
root, whereas `load_static_json!` paths are relative to the file that uses it.

The manager methods receive the request's [`Parts`](https://docs.rs/http/latest/http/request/struct.Parts.html), so
authentication and multi-tenancy can be handled by your own axum middleware or extensions.

## Example

[`examples/smoke.rs`](examples/smoke.rs) implements in-memory `User` and `Group` managers, including filtering, PATCH
and a schema extension:

```sh
cargo run --example smoke
curl localhost:3000/Users
```

## Endpoints

Paths are relative to the SCIM base URL.

| Method | Path | |
|-|-|-|
| `GET` | `/ServiceProviderConfig` | |
| `GET` | `/Schemas`, `/Schemas/{id}` | |
| `GET` | `/ResourceTypes`, `/ResourceTypes/{name}` | |
| `POST` | `/.search` | The filter must include `meta.resourceType eq "<name>"`. |
| `GET` | `<endpoint>` | List and filter resources. |
| `POST` | `<endpoint>/.search` | List and filter resources. |
| `POST` | `<endpoint>` | Create a resource. |
| `GET`, `PUT`, `PATCH`, `DELETE` | `<endpoint>/{id}` | |

## Not supported

- `/Me` and `/Bulk`.
- Searching across resource types from the server root, and `GET /` with query parameters.
- ETags. Cream does not evaluate `If-Match` or `If-None-Match`, but managers can read them from the request `Parts`.

## License

Licensed under either of the [Apache License, Version 2.0](https://www.apache.org/licenses/LICENSE-2.0) or the
[MIT license](https://opensource.org/licenses/MIT), at your option.

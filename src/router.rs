use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{delete, get, patch, post, put},
    Extension, Router,
};

use crate::{error::Error, state::Cream};

mod args;
mod meta;
mod retrieve;
mod update;

impl Cream {
    /// Build an Axum router for the `Cream` instance.
    ///
    /// Routes are relative to the SCIM base URL, so the router can be nested under a prefix:
    ///
    /// - `GET /ServiceProviderConfig`, `GET /Schemas[/{id}]` and `GET /ResourceTypes[/{name}]`
    /// - `POST /.search`, which requires a `meta.resourceType eq "<name>"` filter
    /// - for each resource type's endpoint: `GET` and `POST` on the endpoint, `POST` on
    ///   `<endpoint>/.search`, and `GET`, `PATCH`, `PUT` and `DELETE` on `<endpoint>/{id}`
    pub fn router(&self) -> Router {
        let mut router = meta::router().route("/.search", post(retrieve::search_root));

        for s in self.0.resource_types.values() {
            router = router.nest(
                &s.resource_type.endpoint,
                resource_router(s.resource_type.name.clone()),
            );
        }

        router
            .method_not_allowed_fallback(handle_405)
            .fallback(handle_404)
            .with_state(self.clone())
            .layer(middleware::from_fn_with_state(self.clone(), set_base_url))
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ResourceTypeName(pub String);

#[derive(serde::Deserialize)]
pub(crate) struct ResourceId {
    pub id: String,
}

fn resource_router(resource_type: String) -> Router<Cream> {
    Router::new()
        .route("/", get(retrieve::list_resources))
        .route("/", post(update::create_resource))
        .route("/.search", post(retrieve::search_resources))
        .route("/{id}", get(retrieve::get_resource))
        .route("/{id}", patch(update::patch_resource))
        .route("/{id}", put(update::put_resource))
        .route("/{id}", delete(update::delete_resource))
        .layer(Extension(ResourceTypeName(resource_type)))
}

async fn set_base_url(State(cream): State<Cream>, req: Request, next: Next) -> Response {
    // Set the base URL in task local storage so that we can serialize `Reference`s to
    // absolute URLs.
    cream_core::hidden::BASE_URL
        .scope(cream.0.base_url.clone(), next.run(req))
        .await
}

async fn handle_404() -> impl IntoResponse {
    Error::not_found()
}

async fn handle_405() -> impl IntoResponse {
    Error::new(
        StatusCode::METHOD_NOT_ALLOWED,
        None,
        "Method Not Allowed".to_string(),
    )
}

use axum::{
    error_handling::HandleErrorLayer,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Extension, Router,
};

use crate::{
    error::Error, json::Json, list::ListResponse, resource_type::ResourceTypeName,
    state::CreamState,
};

pub fn router(state: CreamState) -> Router {
    let mut router = Router::new()
        .route("/ServiceProviderConfig", get(get_service_provider_config))
        .route("/Schemas", get(list_schemas))
        .route("/Schemas/:id", get(get_schema))
        .route("/ResourceTypes", get(list_resource_types));

    for resource_type in state.resource_types.values() {
        router = router.nest(
            &resource_type.endpoint,
            resource_router(resource_type.name.clone()),
        );
    }

    router
        .method_not_allowed_fallback(handle_405)
        .fallback(handle_404)
        .with_state(state)
}

fn resource_router(resource_type: String) -> Router<CreamState> {
    Router::new()
        .route("/", get(list_resources))
        .layer(Extension(ResourceTypeName(resource_type)))
}

async fn list_resources(
    State(state): State<CreamState>,
    Extension(ResourceTypeName(resource_type)): Extension<ResourceTypeName>,
) -> impl IntoResponse {
    let resources: Vec<_> = state.resource_types.values().cloned().collect();
    Json(ListResponse {
        total_results: resources.len(),
        items_per_page: resources.len() + 1,
        resources,
        ..Default::default()
    })
}

async fn get_service_provider_config(State(state): State<CreamState>) -> impl IntoResponse {
    let mut config = state.config.clone();
    config.meta.location = Some(format!("{}/ServiceProviderConfig", state.base_url));
    Json(config)
}

async fn list_schemas(State(state): State<CreamState>) -> impl IntoResponse {
    let mut resources: Vec<_> = state.schemas.values().cloned().collect();
    for resource in &mut resources {
        resource.locate(&state.base_url);
    }
    Json(ListResponse {
        total_results: resources.len(),
        items_per_page: resources.len() + 1,
        resources,
        ..Default::default()
    })
}

async fn get_schema(
    State(state): State<CreamState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Error> {
    let mut resource = state.schemas.get(&id).ok_or_else(Error::not_found)?.clone();
    resource.locate(&state.base_url);
    Ok(Json(resource))
}

async fn list_resource_types(State(state): State<CreamState>) -> impl IntoResponse {
    let mut resources: Vec<_> = state.resource_types.values().cloned().collect();
    for resource in &mut resources {
        resource.locate(&state.base_url);
    }
    Json(ListResponse {
        total_results: resources.len(),
        items_per_page: resources.len() + 1,
        resources,
        ..Default::default()
    })
}

async fn get_resource_type(
    State(state): State<CreamState>,
    Path(name): Path<String>,
) -> Result<impl IntoResponse, Error> {
    let mut resource = state
        .resource_types
        .get(&name)
        .ok_or_else(Error::not_found)?
        .clone();
    resource.locate(&state.base_url);
    Ok(Json(resource))
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

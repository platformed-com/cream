use axum::{
    extract::{Path, State},
    response::IntoResponse,
    routing::get,
    Router,
};

use crate::{json::Json, list::ListResponse, Cream, Error};
use cream_core::Reference;

pub(crate) fn router() -> Router<Cream> {
    Router::new()
        .route("/ServiceProviderConfig", get(get_service_provider_config))
        .route("/Schemas", get(list_schemas))
        .route("/Schemas/{id}", get(get_schema))
        .route("/ResourceTypes", get(list_resource_types))
        .route("/ResourceTypes/{name}", get(get_resource_type))
}

async fn get_service_provider_config(State(state): State<Cream>) -> impl IntoResponse {
    let mut config = state.0.config.clone();
    config.meta.location = Some(Reference::new_relative("/ServiceProviderConfig"));
    Json(config)
}

async fn list_schemas(State(state): State<Cream>) -> impl IntoResponse {
    let mut resources: Vec<_> = state.0.schemas.values().cloned().collect();
    for resource in &mut resources {
        resource.locate();
    }
    Json(ListResponse {
        total_results: resources.len(),
        items_per_page: resources.len() + 1,
        resources,
        ..Default::default()
    })
}

async fn get_schema(
    State(state): State<Cream>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Error> {
    let mut resource = state
        .0
        .schemas
        .get(&id)
        .ok_or_else(Error::not_found)?
        .clone();
    resource.locate();
    Ok(Json(resource))
}

async fn list_resource_types(State(state): State<Cream>) -> impl IntoResponse {
    let mut resources: Vec<_> = state
        .0
        .resource_types
        .values()
        .map(|r| r.resource_type.clone())
        .collect();
    for resource in &mut resources {
        resource.locate();
    }
    Json(ListResponse {
        total_results: resources.len(),
        items_per_page: resources.len() + 1,
        resources,
        ..Default::default()
    })
}

async fn get_resource_type(
    State(state): State<Cream>,
    Path(name): Path<String>,
) -> Result<impl IntoResponse, Error> {
    let mut resource = state
        .0
        .resource_types
        .get(&name)
        .ok_or_else(Error::not_found)?
        .resource_type
        .clone();
    resource.locate();
    Ok(Json(resource))
}

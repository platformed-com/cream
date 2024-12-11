use axum::{
    extract::{Path, Query, State},
    http::request::Parts,
    response::IntoResponse,
    Extension,
};
use bumpalo::Bump;

use crate::{
    filter::{self, Visitor as _},
    json::Json,
    list::ListResponse,
    manager,
    state::ResourceTypeState,
    Cream, Error,
};

use super::{
    args::{
        list_optional_attributes, FixAttributeCasingVisitor, GetResourcesArgs, ListResourcesArgs,
    },
    ResourceTypeName,
};

async fn list_resources_inner(
    state: &Cream,
    parts: &Parts,
    resource_type: &str,
    args: ListResourcesArgs,
) -> Result<impl IntoResponse, Error> {
    let rts = state
        .0
        .resource_types
        .get(resource_type)
        .ok_or_else(Error::not_found)?;

    let scope = Bump::new();
    let mut translated_args = manager::ListResourceArgs::default();
    let mut fixer = FixAttributeCasingVisitor::new(&rts.resource_type, state);

    if let Some(filter) = args.filter {
        let filter = scope.alloc(filter::parse_filter(&filter)?);
        // Fix the casing and URNs on any filters
        fixer.visit_filter(filter);

        translated_args.filter = Some(filter.as_ref(&scope));
    }

    if let Some(sort_by) = args.sort_by {
        let sort_by = scope.alloc(filter::parse_attr_path(&sort_by)?);
        // Fix the casing and URNs on any filters
        fixer.visit_attr_path(sort_by);

        translated_args.sort_by = Some(sort_by.as_ref());
    }
    translated_args.sort_order = args.sort_order;

    translated_args.count = args
        .count
        .unwrap_or_else(|| rts.manager.default_page_size());
    translated_args.start_index = args.start_index.unwrap_or(1) - 1;

    let optional_attributes = list_optional_attributes(
        &args.attributes,
        &args.excluded_attributes,
        &mut fixer,
        &scope,
    )?;

    translated_args.optional_attributes = &optional_attributes;

    let result = rts.manager.list(parts, translated_args).await?;
    Ok(Json(ListResponse {
        start_index: args.start_index.unwrap_or(1),
        total_results: result.resources.len(),
        items_per_page: result.items_per_page,
        resources: result.resources,
        ..Default::default()
    }))
}

pub(crate) async fn list_resources(
    State(state): State<Cream>,
    Extension(ResourceTypeName(resource_type)): Extension<ResourceTypeName>,
    Query(args): Query<ListResourcesArgs>,
    parts: Parts,
) -> Result<impl IntoResponse, Error> {
    list_resources_inner(&state, &parts, &resource_type, args).await
}

pub(crate) async fn search_resources(
    State(state): State<Cream>,
    Extension(ResourceTypeName(resource_type)): Extension<ResourceTypeName>,
    parts: Parts,
    Json(args): Json<ListResourcesArgs>,
) -> Result<impl IntoResponse, Error> {
    list_resources_inner(&state, &parts, &resource_type, args).await
}

pub(crate) async fn search_root(
    State(state): State<Cream>,
    parts: Parts,
    Json(args): Json<ListResourcesArgs>,
) -> Result<impl IntoResponse, Error> {
    let scope = Bump::new();
    let mut translated_args = manager::ListResourceArgs::default();

    // Must have a resource type filter
    let filter = args.filter.ok_or_else(Error::invalid_filter)?;

    let (parsed_filter, resource_type) = filter::parse_filter(&filter)?
        .take_resource_type_filter()
        .map_err(|_| Error::invalid_filter())?;

    let rts = state
        .0
        .resource_types
        .get(&resource_type)
        .ok_or_else(Error::not_found)?;

    let mut fixer = FixAttributeCasingVisitor::new(&rts.resource_type, &state);

    if let Some(filter) = parsed_filter {
        let filter = scope.alloc(filter);
        // Fix the casing and URNs on any filters
        fixer.visit_filter(filter);

        translated_args.filter = Some(filter.as_ref(&scope));
    }

    if let Some(sort_by) = args.sort_by {
        let sort_by = scope.alloc(filter::parse_attr_path(&sort_by)?);
        // Fix the casing and URNs on any filters
        fixer.visit_attr_path(sort_by);

        translated_args.sort_by = Some(sort_by.as_ref());
    }
    translated_args.sort_order = args.sort_order;

    translated_args.count = args
        .count
        .unwrap_or_else(|| rts.manager.default_page_size());
    translated_args.start_index = args.start_index.unwrap_or(1) - 1;

    let optional_attributes = list_optional_attributes(
        &args.attributes,
        &args.excluded_attributes,
        &mut fixer,
        &scope,
    )?;

    translated_args.optional_attributes = &optional_attributes;

    let result = rts.manager.list(&parts, translated_args).await?;

    Ok(Json(ListResponse {
        start_index: args.start_index.unwrap_or(1),
        total_results: result.resources.len(),
        items_per_page: result.items_per_page,
        resources: result.resources,
        ..Default::default()
    }))
}

pub(crate) async fn get_resource_inner(
    state: &Cream,
    parts: &Parts,
    rts: &ResourceTypeState,
    args: GetResourcesArgs,
    id: String,
) -> Result<impl IntoResponse, Error> {
    let scope = Bump::new();
    let mut translated_args = manager::GetResourceArgs {
        id,
        optional_attributes: &[],
    };
    let mut fixer = FixAttributeCasingVisitor::new(&rts.resource_type, state);

    let optional_attributes = list_optional_attributes(
        &args.attributes,
        &args.excluded_attributes,
        &mut fixer,
        &scope,
    )?;

    translated_args.optional_attributes = &optional_attributes;

    Ok(Json(rts.manager.get(parts, translated_args).await?))
}

pub(crate) async fn get_resource(
    State(state): State<Cream>,
    Extension(ResourceTypeName(resource_type)): Extension<ResourceTypeName>,
    Path(id): Path<String>,
    Query(args): Query<GetResourcesArgs>,
    parts: Parts,
) -> Result<impl IntoResponse, Error> {
    let rts = state
        .0
        .resource_types
        .get(&resource_type)
        .ok_or_else(Error::not_found)?;

    let result = get_resource_inner(&state, &parts, rts, args, id).await?;
    Ok(result)
}

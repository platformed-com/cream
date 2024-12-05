use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Extension, Router,
};
use bumpalo::Bump;
use serde::Deserialize;

use crate::{
    error::Error,
    filter::{self, default_visit_filter, parse_attr_path, parse_filter, AttrPath, Visitor as _},
    json::Json,
    list::ListResponse,
    manager::{self, SortOrder},
    resource_type::{ResourceType, ResourceTypeName},
    schema::Schema,
    state::Cream,
};

impl Cream {
    pub fn router(&self) -> Router {
        let mut router = Router::new()
            .route("/ServiceProviderConfig", get(get_service_provider_config))
            .route("/Schemas", get(list_schemas))
            .route("/Schemas/:id", get(get_schema))
            .route("/ResourceTypes", get(list_resource_types))
            .route("/ResourceTypes/:name", get(get_resource_type));

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
    }
}

fn resource_router(resource_type: String) -> Router<Cream> {
    Router::new()
        .route("/", get(list_resources))
        .layer(Extension(ResourceTypeName(resource_type)))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ListResourcesArgs {
    // Filtering
    filter: Option<String>,
    // Sorting
    sort_by: Option<String>,
    #[serde(default)]
    sort_order: SortOrder,
    // Pagination
    start_index: Option<usize>,
    count: Option<usize>,
    // Selection
    attributes: Option<String>,
    excluded_attributes: Option<String>,
}

struct FixAttributeCasingVisitor<'a> {
    schema: &'a Schema,
    extension_schemas: Vec<&'a Schema>,
    parent_attr: Option<AttrPath>,
}
impl<'a> FixAttributeCasingVisitor<'a> {
    fn new(resource_type: &'a ResourceType, state: &'a Cream) -> Self {
        Self {
            schema: state
                .0
                .schemas
                .get(&resource_type.schema)
                .expect("Resource type references non-existent schema"),
            extension_schemas: resource_type
                .schema_extensions
                .iter()
                .map(|ext| {
                    state
                        .0
                        .schemas
                        .get(&ext.schema)
                        .expect("Resource type references non-existent schema extension")
                })
                .collect(),
            parent_attr: None,
        }
    }
}

impl filter::Visitor for FixAttributeCasingVisitor<'_> {
    fn visit_filter(&mut self, filter: &mut filter::Filter) {
        match filter {
            filter::Filter::Has(parent, _) => {
                self.parent_attr = Some(parent.clone());
                default_visit_filter(self, filter);
                self.parent_attr = None;
            }
            _ => default_visit_filter(self, filter),
        }
    }
    fn visit_attr_path(&mut self, attr_path: &mut AttrPath) {
        if let Some(attr_urn) = attr_path
            .urn
            .as_ref()
            .or(self.parent_attr.as_ref().and_then(|a| a.urn.as_ref()))
        {
            if attr_urn.eq_ignore_ascii_case(&self.schema.id) {
                self.schema.fix_attribute_casing(
                    attr_path,
                    self.parent_attr.as_ref().map(|a| a.name.as_str()),
                    true,
                );
            } else {
                for extension_schema in &self.extension_schemas {
                    if attr_urn.eq_ignore_ascii_case(&extension_schema.id) {
                        extension_schema.fix_attribute_casing(
                            attr_path,
                            self.parent_attr.as_ref().map(|a| a.name.as_str()),
                            false,
                        );
                        break;
                    }
                }
            }
        } else {
            self.schema.fix_attribute_casing(
                attr_path,
                self.parent_attr.as_ref().map(|a| a.name.as_str()),
                true,
            );
        }
    }
}

#[axum::debug_handler]
async fn list_resources(
    State(state): State<Cream>,
    Extension(ResourceTypeName(resource_type)): Extension<ResourceTypeName>,
    Query(args): Query<ListResourcesArgs>,
) -> Result<impl IntoResponse, Error> {
    let rts = state
        .0
        .resource_types
        .get(&resource_type)
        .ok_or_else(Error::not_found)?;

    let scope = Bump::new();
    let mut translated_args = manager::ListResourceArgs::default();
    let mut fixer = FixAttributeCasingVisitor::new(&rts.resource_type, &state);

    if let Some(filter) = args.filter {
        let filter = scope.alloc(parse_filter(&filter)?);
        // Fix the casing and URNs on any filters
        fixer.visit_filter(filter);

        translated_args.filter = Some(filter.as_ref(&scope));
    }

    if let Some(sort_by) = args.sort_by {
        let sort_by = scope.alloc(parse_attr_path(&sort_by)?);
        // Fix the casing and URNs on any filters
        fixer.visit_attr_path(sort_by);

        translated_args.sort_by = Some(sort_by.as_ref());
    }
    translated_args.sort_order = args.sort_order;

    translated_args.count = args
        .count
        .unwrap_or_else(|| rts.manager.default_page_size());
    translated_args.start_index = args.start_index.unwrap_or(1) - 1;

    let mut include = if let Some(include) = args.attributes {
        include
            .split(',')
            .map(parse_attr_path)
            .collect::<Result<Vec<_>, _>>()?
    } else {
        Vec::new()
    };
    for item in &mut include {
        fixer.visit_attr_path(item);
    }
    let include: Vec<_> = include.iter().map(|a| a.as_ref()).collect();

    let mut exclude = if let Some(exclude) = args.excluded_attributes {
        exclude
            .split(',')
            .map(parse_attr_path)
            .collect::<Result<Vec<_>, _>>()?
    } else {
        Vec::new()
    };
    for item in &mut exclude {
        fixer.visit_attr_path(item);
    }
    let exclude: Vec<_> = exclude.iter().map(|a| a.as_ref()).collect();

    let mut optional_attributes = Vec::new();
    optional_attributes.extend(
        fixer
            .schema
            .list_optional_attributes(&include, &exclude, true),
    );
    for extension in fixer.extension_schemas {
        optional_attributes.extend(extension.list_optional_attributes(&include, &exclude, false));
    }

    translated_args.optional_attributes = &optional_attributes;

    let result = rts.manager.list(translated_args).await?;
    Ok(Json(ListResponse {
        start_index: args.start_index.unwrap_or(1),
        total_results: result.resources.len(),
        items_per_page: result.items_per_page,
        resources: result.resources,
        ..Default::default()
    }))
}

async fn get_service_provider_config(State(state): State<Cream>) -> impl IntoResponse {
    let mut config = state.0.config.clone();
    config.meta.location = Some(format!("{}/ServiceProviderConfig", state.0.base_url));
    Json(config)
}

async fn list_schemas(State(state): State<Cream>) -> impl IntoResponse {
    let mut resources: Vec<_> = state.0.schemas.values().cloned().collect();
    for resource in &mut resources {
        resource.locate(&state.0.base_url);
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
    resource.locate(&state.0.base_url);
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
        resource.locate(&state.0.base_url);
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
    resource.locate(&state.0.base_url);
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

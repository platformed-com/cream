use axum::{
    extract::{Path, Query, State},
    http::{request::Parts, StatusCode},
    response::IntoResponse,
    Extension,
};
use bumpalo::Bump;
use cream_core::SchemaExtension;
use ijson::IObject;

use crate::{
    filter::{self, AttrPathRef, Visitor},
    json::Json,
    manager::{self, UpdateResourceItem},
    Cream, Error,
};

use super::{
    args::{FixAttributeCasingVisitor, GetResourcesArgs, PatchOperationType, PatchResourceArgs},
    retrieve::get_resource_inner,
    ResourceTypeName,
};

fn normalize_update<'a>(
    obj: &'a IObject,
    schema: &'a str,
    extensions: &'a [SchemaExtension],
    is_core: bool,
) -> Vec<UpdateResourceItem<'a>> {
    let mut items = Vec::new();
    'next_attr: for (key, value) in obj {
        if let Some(obj) = value.as_object() {
            for ext in extensions {
                if key.eq_ignore_ascii_case(&ext.schema) {
                    items.extend(normalize_update(obj, &ext.schema, &[], false));
                    continue 'next_attr;
                }
            }
        }
        items.push(UpdateResourceItem {
            path: Some(filter::ValuePathRef::Attr(AttrPathRef {
                urn: if is_core { None } else { Some(schema) },
                name: key,
                sub_attr: None,
            })),
            op: manager::UpdateOp::Replace(value),
        });
    }
    items
}

pub(crate) async fn create_resource(
    State(state): State<Cream>,
    Extension(ResourceTypeName(resource_type)): Extension<ResourceTypeName>,
    Query(args): Query<GetResourcesArgs>,
    parts: Parts,
    Json(body): Json<IObject>,
) -> Result<impl IntoResponse, Error> {
    let rts = state
        .0
        .resource_types
        .get(&resource_type)
        .ok_or_else(Error::not_found)?;
    let id = rts.manager.create(&parts, body).await?;

    get_resource_inner(&state, &parts, rts, args, id)
        .await
        .map(|r| (StatusCode::CREATED, r))
}

pub(crate) async fn patch_resource(
    State(state): State<Cream>,
    Extension(ResourceTypeName(resource_type)): Extension<ResourceTypeName>,
    Path(id): Path<String>,
    Query(args): Query<GetResourcesArgs>,
    parts: Parts,
    Json(body): Json<PatchResourceArgs>,
) -> Result<impl IntoResponse, Error> {
    let scope = Bump::new();
    let rts = state
        .0
        .resource_types
        .get(&resource_type)
        .ok_or_else(Error::not_found)?;
    let mut fixer = FixAttributeCasingVisitor::new(&rts.resource_type, &state);

    let mut items = Vec::new();
    for operation in &body.operations {
        let path = if operation.path.is_empty() {
            None
        } else {
            let value_path = scope.alloc(filter::parse_value_path(&operation.path)?);
            fixer.visit_value_path(value_path);
            Some(value_path.as_ref(&scope))
        };

        // An add or replace at the top level operates field-wise. Translate it to a series of
        // individual updates to make life easier for the manager.
        if path.is_none()
            && matches!(
                operation.op,
                PatchOperationType::Add | PatchOperationType::Replace
            )
        {
            if let Some(obj) = operation.value.as_object() {
                items.extend(normalize_update(
                    obj,
                    &rts.resource_type.schema,
                    &rts.resource_type.schema_extensions,
                    true,
                ));
                continue;
            }
        }

        let op = match operation.op {
            PatchOperationType::Add => manager::UpdateOp::Add(&operation.value),
            PatchOperationType::Remove => manager::UpdateOp::Remove(&operation.value),
            PatchOperationType::Replace => manager::UpdateOp::Replace(&operation.value),
        };
        items.push(UpdateResourceItem { path, op })
    }

    let translated_args = manager::UpdateResourceArgs {
        id: &id,
        items: &items,
    };

    rts.manager.update(&parts, translated_args).await?;

    get_resource_inner(&state, &parts, rts, args, id).await
}

pub(crate) async fn put_resource(
    State(state): State<Cream>,
    Extension(ResourceTypeName(resource_type)): Extension<ResourceTypeName>,
    Path(id): Path<String>,
    Query(args): Query<GetResourcesArgs>,
    parts: Parts,
    Json(body): Json<IObject>,
) -> Result<impl IntoResponse, Error> {
    let rts = state
        .0
        .resource_types
        .get(&resource_type)
        .ok_or_else(Error::not_found)?;
    rts.manager.replace(&parts, &id, body).await?;

    get_resource_inner(&state, &parts, rts, args, id).await
}

pub(crate) async fn delete_resource(
    State(state): State<Cream>,
    Extension(ResourceTypeName(resource_type)): Extension<ResourceTypeName>,
    Path(id): Path<String>,
    parts: Parts,
) -> Result<impl IntoResponse, Error> {
    let rts = state
        .0
        .resource_types
        .get(&resource_type)
        .ok_or_else(Error::not_found)?;
    rts.manager.delete(&parts, &id).await?;
    Ok(StatusCode::NO_CONTENT)
}

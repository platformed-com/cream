use cream::{
    load_static_json, CreamBuilder, Error, GetResourceArgs, ListResourceArgs, ListResourceResult,
    UpdateOp, UpdateResourceArgs,
};
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;

use std::{
    collections::BTreeMap,
    fmt::Debug,
    sync::{Arc, Mutex},
};

mod scim {
    use cream::declare_resource;

    declare_resource!("examples/user_type.json" as User [
        "examples/user_schema.json",
    ]);
}

#[derive(Debug, Default)]
struct ScimManagerState {
    users: BTreeMap<String, scim::User>,
    next_id: u64,
}

#[derive(Debug, Default)]
struct ScimManager(Arc<Mutex<ScimManagerState>>);

#[derive(Default, Debug)]
struct FilterOptions {
    user_name: Option<String>,
    given_name: Option<String>,
    family_name: Option<String>,
    formatted: Option<String>,
    display_name_starts_with: Option<String>,
}

#[axum::async_trait]
impl scim::UserManager for ScimManager {
    async fn list(
        &self,
        args: ListResourceArgs<'async_trait>,
    ) -> Result<ListResourceResult<scim::User>, Error> {
        use cream::filter::prelude::*;

        let mut filter_ops = FilterOptions::default();
        if let Some(filter) = args.filter {
            for expr in filter.iter_cnf() {
                match expr {
                    Compare(scim::User::USER_NAME, Equal | Contains, Str(x)) => {
                        filter_ops.user_name = Some(x.to_string());
                    }
                    Compare(scim::UserName::GIVEN_NAME, Equal | Contains, Str(x)) => {
                        filter_ops.given_name = Some(x.to_string());
                    }
                    Compare(scim::UserName::FAMILY_NAME, Equal | Contains, Str(x)) => {
                        filter_ops.family_name = Some(x.to_string());
                    }
                    Compare(scim::UserName::FORMATTED, Equal | Contains, Str(x)) => {
                        filter_ops.formatted = Some(x.to_string());
                    }
                    Compare(scim::User::DISPLAY_NAME, StartsWith, Str(x)) => {
                        filter_ops.display_name_starts_with = Some(x.to_string());
                    }
                    _ => return Err(Error::invalid_filter()),
                }
            }
        }

        let mut total_count = 0;
        let mut resources = Vec::new();
        for (i, resource) in self
            .0
            .lock()
            .unwrap()
            .users
            .values()
            .filter(|user| {
                if let Some(user_name) = &filter_ops.user_name {
                    if user
                        .user_name
                        .as_ref()
                        .is_none_or(|n| !user_name.eq_ignore_ascii_case(n))
                    {
                        return false;
                    }
                }
                if let Some(given_name) = &filter_ops.given_name {
                    if user.name.as_ref().is_none_or(|n| {
                        n.given_name
                            .as_ref()
                            .is_none_or(|n| !given_name.eq_ignore_ascii_case(n))
                    }) {
                        return false;
                    }
                }
                if let Some(family_name) = &filter_ops.family_name {
                    if user.name.as_ref().is_none_or(|n| {
                        n.family_name
                            .as_ref()
                            .is_none_or(|n| !family_name.eq_ignore_ascii_case(n))
                    }) {
                        return false;
                    }
                }
                if let Some(formatted) = &filter_ops.formatted {
                    if user.name.as_ref().is_none_or(|n| {
                        n.formatted
                            .as_ref()
                            .is_none_or(|n| !formatted.eq_ignore_ascii_case(n))
                    }) {
                        return false;
                    }
                }
                if let Some(display_name_starts_with) = &filter_ops.display_name_starts_with {
                    if user.display_name.as_ref().is_none_or(|n| {
                        !n.to_ascii_lowercase()
                            .starts_with(&display_name_starts_with.to_ascii_lowercase())
                    }) {
                        return false;
                    }
                }
                true
            })
            .enumerate()
        {
            total_count += 1;
            if i >= args.start_index && i < args.start_index + args.count {
                resources.push(resource.clone());
            }
        }

        Ok(ListResourceResult {
            resources,
            total_count,
            items_per_page: args.count,
        })
    }

    async fn get(&self, args: GetResourceArgs<'async_trait>) -> Result<scim::User, Error> {
        self.0
            .lock()
            .unwrap()
            .users
            .get(&args.id)
            .cloned()
            .ok_or_else(Error::not_found)
    }

    async fn create(&self, resource: scim::CreateUser) -> Result<String, Error> {
        let mut guard = self.0.lock().unwrap();
        guard.next_id += 1;
        let id = guard.next_id.to_string();
        guard.users.insert(
            id.clone(),
            scim::User {
                id: id.clone(),
                external_id: resource.external_id,
                user_name: Some(resource.user_name),
                name: resource.name.map(|name| scim::UserName {
                    family_name: name.family_name,
                    given_name: name.given_name,
                    formatted: name.formatted,
                }),
                emails: Some(
                    resource
                        .emails
                        .into_iter()
                        .map(|email| scim::UserEmail {
                            value: email.value,
                            display: email.display,
                            primary: email.primary,
                            type_: email.type_,
                        })
                        .collect(),
                ),
                display_name: resource.display_name,
                groups: None,
                meta: Default::default(),
            },
        );
        Ok(id)
    }

    async fn update(&self, args: UpdateResourceArgs<'async_trait>) -> Result<(), Error> {
        use cream::filter::prelude::*;
        let mut guard = self.0.lock().unwrap();
        let user = guard.users.get_mut(args.id).ok_or_else(Error::not_found)?;
        for item in args.items {
            match (item.path, item.op) {
                (Some(Attr(scim::User::USER_NAME)), UpdateOp::Replace(v)) => {
                    user.user_name = Some(
                        v.as_string()
                            .ok_or_else(|| Error::expected("string"))?
                            .to_string(),
                    );
                }
                (Some(Attr(scim::User::DISPLAY_NAME)), UpdateOp::Replace(v)) => {
                    user.display_name = Some(
                        v.as_string()
                            .ok_or_else(|| Error::expected("string"))?
                            .to_string(),
                    );
                }
                (Some(Attr(scim::UserName::FAMILY_NAME)), UpdateOp::Replace(v)) => {
                    user.name.as_mut().unwrap().family_name = Some(
                        v.as_string()
                            .ok_or_else(|| Error::expected("string"))?
                            .to_string(),
                    );
                }
                (Some(Attr(scim::UserName::GIVEN_NAME)), UpdateOp::Replace(v)) => {
                    user.name.as_mut().unwrap().given_name = Some(
                        v.as_string()
                            .ok_or_else(|| Error::expected("string"))?
                            .to_string(),
                    );
                }
                _ => return Err(Error::invalid_path()),
            }
        }
        Ok(())
    }

    async fn replace(
        &self,
        id: &'async_trait str,
        resource: scim::CreateUser,
    ) -> Result<(), Error> {
        let mut guard = self.0.lock().unwrap();
        guard.users.insert(
            id.into(),
            scim::User {
                id: id.into(),
                external_id: resource.external_id,
                user_name: Some(resource.user_name),
                name: resource.name.map(|name| scim::UserName {
                    family_name: name.family_name,
                    given_name: name.given_name,
                    formatted: name.formatted,
                }),
                emails: Some(
                    resource
                        .emails
                        .into_iter()
                        .map(|email| scim::UserEmail {
                            value: email.value,
                            display: email.display,
                            primary: email.primary,
                            type_: email.type_,
                        })
                        .collect(),
                ),
                display_name: resource.display_name,
                groups: None,
                meta: Default::default(),
            },
        );
        Ok(())
    }
    async fn delete(&self, id: &'async_trait str) -> Result<(), Error> {
        let mut guard = self.0.lock().unwrap();
        guard.users.remove(id);
        Ok(())
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let cream = CreamBuilder::new(
        "http://host.docker.internal:3000",
        load_static_json!("smoke_config.json"),
    )
    .resource_type(scim::User::manage(ScimManager::default()))
    .build();

    // build our application with a single route
    let app = cream
        .router()
        .layer(ServiceBuilder::new().layer(TraceLayer::new_for_http()));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

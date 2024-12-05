use cream::{
    load_static_json, CreamBuilder, Error, GetResourceArgs, ListResourceArgs, ListResourceResult,
    ResourceTypeManager, UpdateResourceArgs,
};
use ijson::IObject;

#[derive(Debug)]
struct UserManager;

#[axum::async_trait]
impl ResourceTypeManager for UserManager {
    async fn list(
        &self,
        args: ListResourceArgs<'async_trait>,
    ) -> Result<ListResourceResult<IObject>, Error> {
        dbg!(&args);
        let resources = vec![];
        let total_count = 0;
        let items_per_page = 0;

        Ok(ListResourceResult {
            resources,
            total_count,
            items_per_page,
        })
    }

    async fn get(
        &self,
        args: GetResourceArgs<'async_trait>,
    ) -> Result<ListResourceResult<IObject>, Error> {
        dbg!(&args);
        let resources = vec![];
        let items_per_page = 0;
        let total_count = 0;

        Ok(ListResourceResult {
            resources,
            items_per_page,
            total_count,
        })
    }

    async fn create(&self, resource: IObject) -> Result<IObject, Error> {
        dbg!(&resource);
        Ok(resource)
    }

    async fn update(&self, args: UpdateResourceArgs<'async_trait>) -> Result<(), Error> {
        dbg!(&args);

        Ok(())
    }
}

#[tokio::main]
async fn main() {
    let cream = CreamBuilder::new(
        "http://localhost:3000",
        load_static_json!("smoke_config.json"),
    )
    .schema(load_static_json!("../static/scim/core/user.json"))
    .schema(load_static_json!("../static/scim/core/group.json"))
    .schema(load_static_json!("../static/scim/enterprise/user.json"))
    .resource_type(load_static_json!("./user_type.json"), UserManager)
    .resource_type(load_static_json!("./group_type.json"), UserManager)
    .build();

    // build our application with a single route
    let app = cream.router();

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

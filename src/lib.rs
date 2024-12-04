use error::Error;
use ijson::IObject;

#[macro_use]
mod error;
mod config;
mod date_time;
mod json;
mod list;
mod macros;
mod meta;
mod resource_type;
mod router;
mod schema;
mod state;

pub struct ListResourceArgs {}

#[axum::async_trait]
pub trait ResourceTypeManager {
    async fn list(&self, args: ListResourceArgs) -> Result<Vec<IObject>, Error>;
}

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}

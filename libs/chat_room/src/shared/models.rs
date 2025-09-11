use async_graphql::SimpleObject;

#[derive(Debug, SimpleObject, Clone)]
pub struct UserNotFound {
    pub name: String,
}

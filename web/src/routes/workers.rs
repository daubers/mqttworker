use std::sync::Arc;
use poem::web::Data;
use poem_openapi::{OpenApi, Tags};
use poem_openapi::payload::PlainText;
use crate::AppState;

pub struct WorkersAPI;

#[derive(Tags)]
enum ApiTags {
    /// Operations about user
    Workers,
}

#[OpenApi]
impl WorkersAPI {
    #[oai(path = "/workers", method = "get", tag = "ApiTags::Workers")]
    async fn workers(&self, state: Data<&Arc<AppState>>) -> PlainText<String> {
        PlainText("hello!".to_string())
    }
}

use std::sync::Arc;
use poem::web::{Data};
use poem_openapi::{OpenApi, Tags};
use poem_openapi::payload::{Json, PlainText, Response};
use crate::AppState;
use mqttboss::models::workers::{Workers as db_workers};

pub struct WorkersAPI;

#[derive(Tags)]
enum ApiTags {
    /// Operations about user
    Workers,
}

#[OpenApi]
impl WorkersAPI {
    #[oai(path = "/workers", method = "get", tag = "ApiTags::Workers")]
    async fn workers(&self, state: Data<&Arc<AppState>>) -> Response<PlainText<String>> {
        let workers = db_workers::read(&mut *state.0.db_connection.lock().await, None);

        println!("{:?}", workers);
        let json_encode = serde_json::to_string(&workers.expect("Failed to read workers")).expect("Failed to encode workers");
        Response::new(PlainText(json_encode)).header("Content-Type", "application/json")
    }
}

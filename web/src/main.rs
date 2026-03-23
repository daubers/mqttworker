pub mod mqtt;
pub mod routes;

use std::sync::Arc;
use futures::lock::Mutex;
use poem::{listener::TcpListener, EndpointExt, Route, Server};
use poem_openapi::{param::Query, payload::PlainText, OpenApi, OpenApiService};
use paho_mqtt::{AsyncClient, Message};
use poem::middleware::AddData;
use poem::web::Data;
use crate::mqtt::connect_client;

struct AppState {
    mqtt_client: Mutex<AsyncClient>,
}

struct Api;

#[OpenApi]
impl Api {
    #[oai(path = "/hello", method = "get")]
    async fn index(&self, name: Query<Option<String>>, state: Data<&Arc<AppState>>) -> PlainText<String> {
        let client = state.0.mqtt_client.lock().await;
        client.publish(Message::new("/test", "Hi", 1)).await.unwrap();
        match name.0 {
            Some(name) => PlainText(format!("hello, {name}!")),
            None => PlainText("hello!".to_string()),
        }
    }

}



#[tokio::main]
async fn main() -> Result<(), std::io::Error> {

    let state = Arc::new(AppState {
        mqtt_client: Mutex::new(connect_client().await.unwrap()),
    });
    let all_endpoints = (routes::workers::WorkersAPI, Api);
    let api_service =
        OpenApiService::new(all_endpoints, "Hello World", "1.0")
            .server("http://localhost:3000/api");
    let ui = api_service.swagger_ui();
    

    Server::new(TcpListener::bind("0.0.0.0:3000"))
        .run(Route::new()
            .nest("/api", api_service)
            .nest("/", ui)
            .with(AddData::new(state)))
        .await
}
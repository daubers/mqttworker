mod models;
mod schema;
pub mod messaging;

use crate::messaging::workers;
use std::{env, thread};
use ::diesel::{Connection, PgConnection};
use dotenvy::dotenv;
use paho_mqtt as mqtt;
use serde_json::Value;
use mqttworker::messages::process_message as get_message;

pub mod diesel {
    pub use diesel::*;
}

pub fn establish_connection() -> PgConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

fn main() {
    let hostname = "localhost";
    let client_id = "boss";
    let topic = "workers/#".to_string();
    let qos = 1;
    let username = "RustClient";
    let password = "RustPwd";

    // Create a client creation option object. This is used to pass further information during the client creation process.
    let client_options = mqtt::CreateOptionsBuilder::new()
        .server_uri(hostname)
        .client_id(client_id)
        .finalize();

    // Create the MQTT client
    let client = mqtt::Client::new(client_options).expect("Error during client creation");

    // Create a connection option object to configure the username and other information.
    let connection_options = mqtt::ConnectOptionsBuilder::new()
        .clean_session(true)
        .finalize();
    client.connect(connection_options).expect("Failed to connect to broker");
    client.subscribe(&topic, qos).expect("Failed to subscribe");

    // Starts the client receiving messages
    let rx_queue = client.start_consuming();
    // Create a thread that stays pending over incoming messages.
    let handle = thread::spawn(move || {
        let connection = &mut establish_connection();
        for mqttmsg in rx_queue.iter() {
            if let Some(mqttmsg) = mqttmsg {
                println!("Received: -> {:?}", get_message(&mqttmsg).unwrap());
                println!("Received: -> {}", mqttmsg.topic());
            } else {
                println!("Unsubscribe: connection closed");
                break;
            }
        }
    });

    loop {
        thread::sleep(std::time::Duration::from_secs(1));
    }
    println!("Hello, world!")   ;
}

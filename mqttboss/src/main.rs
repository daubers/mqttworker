mod models;
mod schema;
pub mod messaging;
pub mod db;

use std::thread;
use paho_mqtt as mqtt;
use mqttworker::messages::process_message as get_message;
use crate::messaging::workers::process_message;

pub mod diesel {
    pub use diesel::*;
}



fn main() {
    let hostname = "localhost";
    let client_id = "boss";
    let topic = "workers/#".to_string();
    let qos = 1;
    let _username = "RustClient";
    let _password = "RustPwd";

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
    let _handle = thread::spawn(move || {
        for mqttmsg in rx_queue.iter() {
            if let Some(mqttmsg) = mqttmsg {
                println!("Unwrapped message: -> {:?}", get_message(&mqttmsg).unwrap());
                println!("Received: -> {}", mqttmsg.topic());
                process_message(mqttmsg);
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

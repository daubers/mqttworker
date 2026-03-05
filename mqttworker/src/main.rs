pub mod messages;

use paho_mqtt as mqtt;
use std::{thread, time::Duration};
use crate::messages::{CapabilitiesMessage, WorkerAnnouncement};

fn main() {
    let hostname = "localhost";
    let client_id = "rust_cedalo_client";
    let topic = "#";
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

    // Connect to the MQTT broker
    client.connect(connection_options).expect("Failed to connect to broker");

    // Subscribe to the topic
    client.subscribe(topic, qos).expect("Failed to subscribe");

    // Starts the client receiving messages
    let rx_queue = client.start_consuming();
    // Create a thread that stays pending over incoming messages.
    let handle = thread::spawn(move || {
        for mqttmsg in rx_queue.iter() {
            if let Some(mqttmsg) = mqttmsg {
                println!("Received: -> {}", mqttmsg.payload_str());
            } else {
                println!("Unsubscribe: connection closed");
                break;
            }
        }
    });

    let wa = WorkerAnnouncement::new(client.clone(), "test_worker".to_string(), 10);
    wa.run();

    // Publish a message
    let testmsg = CapabilitiesMessage::new("test_worker".to_string());
    let mqttmsg = mqtt::Message::new(testmsg.topic(), testmsg.message(), qos);
    client.publish(mqttmsg).expect("Failed to publish message");

    // Keep the program alive for a few seconds to receive messages
    thread::sleep(Duration::from_secs(10));

    // Disconnect the client
    client.disconnect(None).expect("Failed to disconnect");
    handle.join().expect("Failed to join thread");
    println!("Disconnected");
}
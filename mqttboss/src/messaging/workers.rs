use paho_mqtt::Message;
use mqttworker::messages::process_message as get_message;
use mqttworker::MessageType;
use crate::models::workers::{CreateWorkers, Workers};
use crate::db::establish_connection;

pub fn process_message(message: Message) {
    let connection = &mut establish_connection();
    match get_message(&message) {
        Some(message_struct) => {
            match message_struct {
                MessageType::Capabilities(_msg) => {

                }
                MessageType::Announcement(msg) => {
                    let db_results =  Workers::search_by_message(connection, &msg).unwrap();
                    if db_results.is_empty() {
                        // create new record
                        Workers::create(connection, &CreateWorkers { id: None, name: msg.message_config.worker_id, capabilities: None, last_seen: None }).unwrap();
                    }
                    println!("DBResults: -> {:?}", db_results);
                }
            }
        },
        None => todo!(),
    }
}
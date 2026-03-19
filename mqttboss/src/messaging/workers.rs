use chrono::Utc;
use paho_mqtt::Message;
use mqttworker::messages::process_message as get_message;
use mqttworker::{MessageType, WorkerAnnouncementType};
use crate::models::workers::{CreateWorkers, UpdateWorkers, Workers};
use crate::db::establish_connection;

pub fn process_message(message: Message) {
    let connection = &mut establish_connection();
    match get_message(&message) {
        Some(message_struct) => {
            match message_struct {
                MessageType::Capabilities(_msg) => {

                }
                MessageType::Announcement(msg) => {
                    let available = match msg.announcement_type {
                        WorkerAnnouncementType::Online => {true}
                        WorkerAnnouncementType::ShutdownUnexpected => {false}
                        WorkerAnnouncementType::ShutdownExpected => {false}
                    };
                    let db_results =  Workers::search_by_message(connection, &msg).unwrap();
                    if db_results.is_empty() {
                        // create new record
                        Workers::create(connection, &CreateWorkers { id: None, name: msg.message_config.worker_id, last_seen: None, cpus: None, ram: None, disk: None, gpu: None, tags: None, available: available }).unwrap();
                    } else {
                        for item in db_results {
                            println!("Worker found: {:?}", item);
                            Workers::update(connection, item.id, &UpdateWorkers { name: Some(msg.message_config.worker_id.clone()), last_seen: Some(Option::from(Utc::now().naive_local())), cpus: None, ram: None, disk: None, gpu: None, tags: None, available: available }).unwrap();
                        }
                    }
                }
            }
        },
        None => todo!(),
    }
}
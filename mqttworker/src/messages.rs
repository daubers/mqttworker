use std::{fmt, thread};
use std::fmt::Formatter;
use sysinfo::{
    Components, Disks, Networks, System,
};
use serde::{Deserialize, Serialize};
use serde_json;
use paho_mqtt as mqtt;
use paho_mqtt::Client;
use crate::messages::MessageType::{Announcement, Capabilities};

#[derive(Serialize, Deserialize, Debug)]
enum MessageDirection {
    Request,
    Response,
    Broadcast,
}



#[derive(Serialize, Deserialize, Debug)]
pub struct Message {
    pub direction: MessageDirection,
    pub worker_id: String,
    pub message_type: String,
    pub topic: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CapabilitiesMessage {
    message_config: Message,
    available_memory: u64,
    num_cores: u32,
    architecture: String,
}

impl CapabilitiesMessage {
    pub fn new(worker_id: String) -> CapabilitiesMessage {
        // Please note that we use "new_all" to ensure that all lists of
        // CPUs and processes are filled!
        let mut sys = System::new_all();

        // First we update all information of our `System` struct.
        sys.refresh_all();

        CapabilitiesMessage {
            message_config: Message {worker_id, direction: MessageDirection::Broadcast,message_type: "capabilities".to_string(), topic: "workers/capabilities".to_string() },
            available_memory: sys.total_memory(),
            num_cores: sys.cpus().len() as u32,
            architecture: std::env::consts::ARCH.to_string(),
        }
    }

    pub fn message(&self) -> String {
        serde_json::to_string(self).unwrap()
    }

    pub fn topic(&self) -> String {
        self.message_config.topic.clone()
    }
}

fn mqtt_client_default() -> Client {
    let client_options = mqtt::CreateOptionsBuilder::new()
        .finalize();
    mqtt::Client::new(client_options).unwrap()
}

#[derive(Serialize, Deserialize)]
pub struct WorkerAnnouncement {
    pub message_config: Message,
    #[serde(skip)]
    #[serde(default = "mqtt_client_default")]
    mqtt_client: mqtt::Client,
    pub broadcast_interval: u64
}

impl fmt::Debug for WorkerAnnouncement {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("WorkerAnnouncement")
            .field("message_config", &self.message_config)
            .field("broadcast_interval", &self.broadcast_interval)
            .finish()
    }
}

impl WorkerAnnouncement {
    pub fn new(mqtt_client: mqtt::Client, worker_id: String, broadcast_interval: u64) -> WorkerAnnouncement {
        WorkerAnnouncement {
            message_config: Message {worker_id, direction: MessageDirection::Broadcast,message_type: "worker_announcement".to_string(), topic: "workers/announcements".to_string() },
            mqtt_client,
            broadcast_interval
        }
    }

    pub fn run(&self) {
        let announcement_message = serde_json::to_string(&self).unwrap();
        let message = mqtt::Message::new(self.message_config.topic.clone(), announcement_message, 1);
        let mqtt_client = self.mqtt_client.clone();
        let broadcast_interval = self.broadcast_interval.clone();
        let broadcast_thread_result = thread::Builder::new().name("worker_announcer".to_string()).spawn(move || {
            loop {
                match mqtt_client.publish(message.clone()) {
                    Ok(_) => (),
                    Err(e) => {}
                };
                thread::sleep(std::time::Duration::from_secs(broadcast_interval));
            }
        });
        let broadcast_thread = match broadcast_thread_result {
            Ok(broadcast_thread) => {
                broadcast_thread
            },
            Err(e) => {
                println!("Failed to spawn broadcast thread: {}", e);
                return;
            }
        };
    }
}

#[derive(Debug)]
pub enum MessageType {
    Capabilities(CapabilitiesMessage),
    Announcement(WorkerAnnouncement),
}

pub fn process_message(message: &mqtt::Message) -> Option<MessageType> {
    let deserialized_message = match message.topic() {
        "workers/announcements" => {
            let this_message: WorkerAnnouncement = serde_json::from_str(message.payload_str().to_string().as_str()).unwrap();
            Some(Announcement(this_message))
        },
        "workers/capabilities" => {
            let this_message: CapabilitiesMessage = serde_json::from_str(message.payload_str().to_string().as_str()).unwrap();
            Some(Capabilities(this_message))
        },
        &_ => {
            None
        }
    };
    deserialized_message
}

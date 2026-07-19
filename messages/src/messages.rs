use std::{fmt, thread};
use std::fmt::Formatter;
use sysinfo::{
    System,
};
use serde::{Deserialize, Serialize};
use serde_json;
use paho_mqtt as mqtt;
use paho_mqtt::Client;
use uuid::Uuid;
use crate::messages::MessageType::{Announcement, Capabilities, WorkerRequest};

#[derive(Serialize, Deserialize, Debug)]
pub enum MessageDirection {
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
    pub msg_id: Option<uuid::Uuid>,
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
            message_config: Message {worker_id, direction: MessageDirection::Broadcast,message_type: "capabilities".to_string(), topic: "workers/capabilities".to_string(), msg_id: Some(Uuid::new_v4()) },
            available_memory: sys.total_memory(),
            num_cores: sys.cpus().len() as u32,
            architecture: std::env::consts::ARCH.to_string(),
        }
    }

    pub fn message(&self) -> paho_mqtt::Message {
        mqtt::Message::new(self.message_config.topic.clone(), serde_json::to_string(self).unwrap(), 1)
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

#[derive(Serialize, Deserialize, Debug)]
pub enum WorkerAnnouncementType {
    Online,
    ShutdownUnexpected,
    ShutdownExpected
}

#[derive(Serialize, Deserialize)]
pub struct WorkerAnnouncement {
    pub message_config: Message,
    #[serde(skip)]
    mqtt_client: Option<mqtt::Client>,
    pub broadcast_interval: Option<u64>,
    pub announcement_type: WorkerAnnouncementType,
}

impl fmt::Debug for WorkerAnnouncement {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("WorkerAnnouncement")
            .field("message_config", &self.message_config)
            .field("broadcast_interval", &self.broadcast_interval)
            .field("announcement_type", &self.announcement_type)
            .finish()
    }
}

impl WorkerAnnouncement {
    pub fn new(mqtt_client: Option<mqtt::Client>, worker_id: String, announcement_type: WorkerAnnouncementType, broadcast_interval: Option<u64>) -> WorkerAnnouncement {
        WorkerAnnouncement {
            message_config: Message {worker_id, direction: MessageDirection::Broadcast,message_type: "worker_announcement".to_string(), topic: "workers/announcements".to_string(), msg_id: Some(Uuid::new_v4()) },
            mqtt_client,
            broadcast_interval,
            announcement_type
        }
    }
    pub fn message(&self) -> paho_mqtt::Message {
        mqtt::Message::new(self.message_config.topic.clone(), serde_json::to_string(self).unwrap(), 1)
    }

    pub fn run(&self) {
        match self.broadcast_interval {
            None => {}
            Some(broadcast_interval) => {
                let announcement_message = serde_json::to_string(&self).unwrap();
                let message = mqtt::Message::new(self.message_config.topic.clone(), announcement_message, 1);
                let mqtt_client = self.mqtt_client.as_ref().expect("No client available").clone();
                let broadcast_thread_result = thread::Builder::new().name("worker_announcer".to_string()).spawn(move || {
                    loop {
                        match mqtt_client.publish(message.clone()) {
                            Ok(_) => (),
                            Err(_e) => {}
                        };
                        thread::sleep(std::time::Duration::from_secs(broadcast_interval));
                    }
                });
                let _broadcast_thread = match broadcast_thread_result {
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
        }

}

#[derive(Serialize, Deserialize)]
pub struct WorkerRequestRunJob {}

#[derive(Serialize, Deserialize)]
pub struct WorkerRequestJobStatus {
    job_id: Uuid,
}

#[derive(Serialize, Deserialize)]
pub struct WorkerRequestShutdown {}

#[derive(Serialize, Deserialize)]
pub enum WorkerRequestQueryType {
    RunJob(WorkerRequestRunJob),
    JobStatus(WorkerRequestJobStatus),
    Shutdown(WorkerRequestShutdown)
}

#[derive(Serialize, Deserialize)]
pub struct WorkerRequestQuery {
    pub message_config: Message,
    pub query_type: WorkerRequestQueryType,

    #[serde(skip)]
    #[serde(default = "mqtt_client_default")]
    mqtt_client: mqtt::Client,
}

#[derive(Serialize, Deserialize)]
pub struct WorkerRequestMessage {
    pub message_config: Message,
    pub query: String,
    #[serde(skip)]
    #[serde(default = "mqtt_client_default")]
    mqtt_client: mqtt::Client,
}

impl fmt::Debug for WorkerRequestMessage {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("WorkerRequest")
            .field("message_config", &self.message_config)
            .field("query", &self.query)
            .finish()
    }
}

impl WorkerRequestMessage {
    pub fn new(mqtt_client: &Client, worker_id: String, msg_id: Uuid, query: String) -> WorkerRequestMessage {
        WorkerRequestMessage {
            message_config: Message {
                direction: MessageDirection::Request,
                worker_id,
                message_type: "".to_string(),
                topic: "/workers/request".to_string(),
                msg_id: Some(msg_id),
            },
            query,
            mqtt_client: mqtt_client.clone(),
        }
    }
    pub fn message(&self) -> paho_mqtt::Message {
        mqtt::Message::new(self.message_config.topic.clone(), serde_json::to_string(self).unwrap(), 1)
    }

}

#[derive(Debug)]
pub enum MessageType {
    Capabilities(CapabilitiesMessage),
    Announcement(WorkerAnnouncement),
    WorkerRequest(WorkerRequestMessage),
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
        "workers/request" => {
            let this_message: WorkerRequestMessage = serde_json::from_str(message.payload_str().to_string().as_str()).unwrap();
            Some(WorkerRequest(this_message))
        },
        &_ => {
            None
        }
    };
    deserialized_message
}

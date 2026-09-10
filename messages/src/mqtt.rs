use std::env;
use std::sync::Arc;
use paho_mqtt as mqtt;
use paho_mqtt::{AsyncReceiver, Message};
use serde::{Deserialize, Serialize};
use crate::messages::{WorkerAnnouncement, WorkerAnnouncementType};

pub struct ConnectedClient {
    pub client: mqtt::AsyncClient,
    pub message_stream: AsyncReceiver<Option<Message>>,
}

pub async fn connect_client_async(broker: Broker, send_online: bool) -> Result<ConnectedClient, paho_mqtt::Error> {
    let mqtt_create_opts = mqtt::CreateOptionsBuilder::new()
        .server_uri(broker.broker_uri.clone())
        .mqtt_version(paho_mqtt::MqttVersion::V5)
        .finalize();

    // Create the client
    let mut cli = mqtt::AsyncClient::new(mqtt_create_opts).expect("Can't create client");

    // create the stream
    let strm = cli.get_stream(25);
    // Connect with default options and wait for it to complete or fail
    // The default is an MQTT v3.x connection.
    if broker.broker_authenticate {
        let mqtt_connect_options = mqtt::ConnectOptionsBuilder::new_v5()
            .user_name(broker.credentials.clone().unwrap().username.clone())
            .password(broker.credentials.clone().unwrap().password.clone())
            .finalize();
        cli.connect(mqtt_connect_options).await.expect("Can't connect");
    } else {
        let mqtt_connect_options = mqtt::ConnectOptionsBuilder::new_v5().finalize();
        cli.connect(mqtt_connect_options).await.expect("Can't connect");
    }
    if send_online {
        let msg = WorkerAnnouncement::new(None, "worker_tmp_id".to_string(), WorkerAnnouncementType::Online, None);
        cli.publish(msg.message()).await.expect("Can't publish");
    }
    Ok(ConnectedClient {client:cli, message_stream: strm})
}

pub fn connect_client_sync(send_online: bool, broker: &Broker) -> Result<mqtt::Client, paho_mqtt::Error> {
    let mqtt_create_opts = mqtt::CreateOptionsBuilder::new()
        .server_uri(broker.broker_uri.clone())
        .mqtt_version(paho_mqtt::MqttVersion::V5)
        .finalize();

    // Create the client
    let mut cli = mqtt::Client::new(mqtt_create_opts).expect("Can't create client");

    // Connect with default options and wait for it to complete or fail
    // The default is an MQTT v3.x connection.
    if broker.broker_authenticate {
        let mqtt_connect_options = mqtt::ConnectOptionsBuilder::new_v5()
            .user_name(broker.credentials.clone().unwrap().username.clone())
            .password(broker.credentials.clone().unwrap().password.clone())
            .finalize();
        cli.connect(mqtt_connect_options).expect("Can't connect");
    } else {
        let mqtt_connect_options = mqtt::ConnectOptionsBuilder::new_v5().finalize();
        cli.connect(mqtt_connect_options).expect("Can't connect");
    }
    if send_online {
        let msg = WorkerAnnouncement::new(None, "worker_tmp_id".to_string(), WorkerAnnouncementType::Online, None);
        cli.publish(msg.message()).expect("Can't publish");
    }
    Ok(cli)
}

// Configuration stuff
#[derive(Serialize, Deserialize, Clone)]
pub struct Credentials {
    pub username: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Broker {
    pub broker_uri: String, // "mqtt://localhost:1883"
    #[serde(default="default_broker_auth")]
    pub broker_authenticate: bool,
    #[serde(default="default_credentials")]
    pub credentials: Option<Credentials>,
    pub topics: Vec<String>
}

fn default_broker_auth() -> bool {
    true
}

fn default_credentials() -> Option<Credentials> {
    None
}
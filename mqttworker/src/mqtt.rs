use std::env;
use std::sync::Arc;
use paho_mqtt as mqtt;
use messages::messages::{WorkerAnnouncement, WorkerAnnouncementType};
use crate::configuration::Config;

pub async fn connect_client_async(configuration: Arc<Config>, send_online: bool) -> Result<mqtt::AsyncClient, paho_mqtt::Error> {
    // Command-line option(s)
    let mqtt_connect_options = mqtt::ConnectOptionsBuilder::new()
        .user_name(configuration.broker.credentials.clone().unwrap().username.clone())
        .password(configuration.broker.credentials.clone().unwrap().password.clone())
        .finalize();

    let mqtt_create_opts = mqtt::CreateOptionsBuilder::new()
        .server_uri(configuration.broker.broker_uri.clone())
        .finalize();

    // Create the client
    let cli = mqtt::AsyncClient::new(mqtt_create_opts).expect("Can't create client");

    // Connect with default options and wait for it to complete or fail
    // The default is an MQTT v3.x connection.
    if configuration.broker.broker_authenticate {
        cli.connect(mqtt_connect_options).await.expect("Can't connect");
    } else {
        cli.connect(None).await.expect("Can't connect");
    }
    if send_online {
        let msg = WorkerAnnouncement::new(None, "worker_tmp_id".to_string(), WorkerAnnouncementType::Online, None);
        cli.publish(msg.message()).await.expect("Can't publish");
    }
    Ok(cli)
}

pub fn connect_client_sync(send_online: bool) -> Result<mqtt::Client, paho_mqtt::Error> {
    // Command-line option(s)
    let host = env::args()
        .nth(1)
        .unwrap_or_else(|| "mqtt://localhost:1883".to_string());

    // Create the client
    let cli = mqtt::Client::new(host).expect("Can't create client");

    // Connect with default options and wait for it to complete or fail
    // The default is an MQTT v3.x connection.
    let cli = match cli.connect(None) {
        Ok(_) => cli,
        Err(err) => {
            println!("Error connecting to MQTT broker: {:#?}", err);
            cli
        },
    };
    if send_online && cli.is_connected() {
        let msg = WorkerAnnouncement::new(None, "worker_tmp_id".to_string(), WorkerAnnouncementType::Online, None);
        cli.publish(msg.message()).expect("Can't publish");
    }
    Ok(cli)
}
use std::env;
use std::sync::Arc;
use paho_mqtt as mqtt;
use paho_mqtt::{AsyncReceiver, Message};
use messages::messages::{WorkerAnnouncement, WorkerAnnouncementType};
use crate::configuration::Config;

pub struct ConnectedClient {
    pub(crate) client: mqtt::AsyncClient,
    pub(crate) message_stream: AsyncReceiver<Option<Message>>,
}

pub async fn connect_client_async(configuration: Arc<Config>, send_online: bool) -> Result<ConnectedClient, paho_mqtt::Error> {
    let mqtt_create_opts = mqtt::CreateOptionsBuilder::new()
        .server_uri(configuration.broker.broker_uri.clone())
        .mqtt_version(paho_mqtt::MqttVersion::V5)
        .finalize();

    // Create the client
    let mut cli = mqtt::AsyncClient::new(mqtt_create_opts).expect("Can't create client");

    // create the stream
    let strm = cli.get_stream(25);
    // Connect with default options and wait for it to complete or fail
    // The default is an MQTT v3.x connection.
    if configuration.broker.broker_authenticate {
        let mqtt_connect_options = mqtt::ConnectOptionsBuilder::new_v5()
            .user_name(configuration.broker.credentials.clone().unwrap().username.clone())
            .password(configuration.broker.credentials.clone().unwrap().password.clone())
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
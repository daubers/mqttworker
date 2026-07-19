use std::env;
use paho_mqtt as mqtt;
use messages::messages::{WorkerAnnouncement, WorkerAnnouncementType};

pub async fn connect_client_async(send_online: bool) -> Result<mqtt::AsyncClient, paho_mqtt::Error> {
    // Command-line option(s)
    let host = env::args()
        .nth(1)
        .unwrap_or_else(|| "mqtt://localhost:1883".to_string());

    // Create the client
    let cli = mqtt::AsyncClient::new(host).expect("Can't create client");

    // Connect with default options and wait for it to complete or fail
    // The default is an MQTT v3.x connection.
    cli.connect(None).await.expect("Can't connect");
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
    cli.connect(None).expect("Can't connect");
    if send_online {
        let msg = WorkerAnnouncement::new(None, "worker_tmp_id".to_string(), WorkerAnnouncementType::Online, None);
        cli.publish(msg.message()).expect("Can't publish");
    }
    Ok(cli)
}
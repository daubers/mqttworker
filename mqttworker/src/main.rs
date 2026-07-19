pub mod mqtt;

use tokio;
use tokio::signal::unix::{signal, SignalKind};
use tokio_cron_scheduler::{Job, JobScheduler};
use messages::messages::CapabilitiesMessage;
use crate::mqtt::{connect_client_async, connect_client_sync};

#[tokio::main]
async fn main() {
    let mqtt_client = connect_client_async(true).await.unwrap();
    let mut scheduler = JobScheduler::new().await.expect("Can't initialise scheduler");
    scheduler.add(
        Job::new_async("1/5 * * * * *", |uuid, mut _l| {
            let mqtt_client_c = connect_client_sync(false).expect("Can't connect sync mqtt client");
            Box::pin(async move {
                let capabilities_announcement = CapabilitiesMessage::new("test".to_string());
                mqtt_client_c.publish(capabilities_announcement.message()).expect("Can't publish message");
                mqtt_client_c.disconnect(None).expect("Can't disconnect");
            })
        }
        ).expect("Can't add job")
    ).await.expect("Can't add job");

    // Start the scheduler
    scheduler.start().await.expect("Can't start scheduler");

    let mut sig_int_handler = signal(SignalKind::interrupt()).expect("Can't create signal");
    let mut sig_term_handler = signal(SignalKind::terminate()).expect("Can't create signal");
    let mut sig_hup_handler = signal(SignalKind::hangup()).expect("Can't create signal");
    // Wait while the jobs run
    tokio::select! {
        _ = sig_int_handler.recv() => println!("SIGINT"),
        _ = sig_term_handler.recv() => println!("SIGTERM"),
        _ = sig_hup_handler.recv() => println!("SIGHUP"),
    }
    println!("terminating the process...");
    let _ = scheduler.shutdown();
    mqtt_client.disconnect(None).await.expect("Can't disconnect client");
}
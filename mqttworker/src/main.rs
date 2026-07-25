pub mod mqtt;
pub mod configuration;

use std::sync::Arc;
use clap::Parser;

use tokio;
use tokio::signal::unix::{signal, SignalKind};
use tokio_cron_scheduler::{Job, JobScheduler};
use messages::messages::CapabilitiesMessage;
use crate::configuration::Config;
use crate::mqtt::connect_client_async;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[arg(short, long)]
    configuration_file: String,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let configuration = Arc::new(Config::new(&args.configuration_file));
    let mqtt_client = connect_client_async(configuration, true).await.unwrap();
    let mut scheduler = JobScheduler::new().await.expect("Can't initialise scheduler");
    let conf_file = args.configuration_file.clone();
    scheduler.add(
        Job::new_async("1/5 * * * * *", move |_uuid, mut _l| {
            Box::pin({
            {
            let value = conf_file.clone();
            async move {
                let conf = Arc::new(Config::new(value.as_str()));
                let mqtt_client_c = connect_client_async(conf.clone(), false).await.expect("Can't connect sync mqtt client");

                let node_name = conf.node_name.clone().as_str().to_string();
                let capabilities_announcement = CapabilitiesMessage::new(node_name);
                mqtt_client_c.publish(capabilities_announcement.message()).await.expect("Can't publish message");
                mqtt_client_c.disconnect(None).await.expect("Can't disconnect");
            }}})
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
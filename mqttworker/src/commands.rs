use std::sync::Arc;
use crate::configuration::{JobDefinition, Task};
use crate::containers::run_task;
use messages::messages::WorkerStartJobMessage;
use paho_mqtt::Message;
use regex::Captures;
use messages::mqtt::ConnectedClient;

pub async fn start_job(msg: &Message, cmd: Captures<'_>, mqttc: Arc<ConnectedClient>) {
    let command = cmd.name("cmd").unwrap().as_str();
    println!("Worker command: {:#?}", command);
    match command {
        "startjob" => {
            println!("Starting job");
            let job_full: WorkerStartJobMessage = serde_json::from_str(msg.payload_str().to_string().as_str()).unwrap();
            let job_definition = JobDefinition::from_str(job_full.workflow.as_str());
            let tasks: Task = match job_definition.job.first_key_value() {
                None => { panic!() }
                Some((_k, v)) => v.clone()[0].clone()
            };
            run_task(tasks, mqttc, None).await;
        }
        "stopjob" => { println!("Stopping job") }
        _ => { println!("Unknown worker command") }
    }
}
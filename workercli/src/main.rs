mod configuration;

use std::{env, fs};
use std::env::VarError;
use clap::{Args, Parser, Subcommand};
use messages::messages::{WorkerRequestMessage, WorkerStartJobMessage};
use messages::mqtt::{Broker, Credentials};
use crate::configuration::CliConfig;

#[derive(Debug, Parser)] // requires `derive` feature
#[command(name = "Worker CLI")]
#[command(about = "CLI for dealing with worker jobs", long_about = None)]
struct Cli {
    #[clap(flatten)]
    global_opts: GlobalOpts,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Args)]
struct GlobalOpts {
    #[arg(long, short, env = "MQTTWORKER_CLIENT_CONFIG_PATH", default_value = "~/.mqttworker/cli_config.yaml")]
    configuration_file_path: String,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Job {
        #[command(subcommand)]
        job_commands: JobCommands,

    }
}

#[derive(Debug, Subcommand)]
enum JobCommands {
    #[command(name = "run")]
    Run {
        //Path to the Job yaml file
        #[arg(long, short)]
        job_file_path: String,
        // Follow the output from the job
        #[arg(long, short)]
        follow: bool,
    },
}


fn start_job( job_file_path: String){
    println!("Starting job");
}

fn load_broker_config(path_to_config: Option<String>) -> messages::mqtt::Broker {
     match path_to_config {
        None => {
            let credentials = match env::var("MQTT_USERNAME") {
                Ok(_) => {Some(Credentials {
                    username: env::var("MQTT_USERNAME").unwrap_or("Missing MQTT_USERNAME env variable".to_string()),
                    password: env::var("MQTT_PASSWORD").unwrap_or("Missing MQTT_PASSWORD env variable".to_string()),
                })}
                Err(_) => {None}
            };
            Broker {
                broker_uri: env::var("MQTT_BROKER_URI").unwrap_or("Missing MQTT_BROKER_URI env variable".to_string()),
                broker_authenticate: env::var("MQTT_AUTHENTICATION").is_ok(),
                credentials,
                topics: vec![],
                }
            }
        Some(path) => {
            let file_contents = fs::read_to_string(path).expect("Can't read config file");
            let config: Broker = toml::from_str(file_contents.as_str()).expect("Can't parse config file");
            config
        }
    }
}

fn main() {
    let cli = Cli::parse();
    let config = CliConfig::from_file(&cli.global_opts.configuration_file_path);
    let mqttc = messages::mqtt::connect_client_sync(false, &config.broker).expect("Can't connect sync mqtt client");
    match cli.command {
        Commands::Job { job_commands } => {
            match job_commands {
                JobCommands::Run { job_file_path, .. } => {
                    let job: String = match fs::read_to_string(job_file_path){
                        Ok(job) => job,
                        Err(err) => {
                            println!("Can't read job file: {}", err);
                            std::process::exit(1);
                        }
                    };
                    let message = WorkerStartJobMessage::new(&mqttc, config.client_name,"sample1".to_string() , uuid::Uuid::new_v4(), job);
                    match mqttc.publish(message.message().clone()) {
                        Ok(_) => (),
                        Err(_e) => {}
                    };
                }
            }
        }
    }
}
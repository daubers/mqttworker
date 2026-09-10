use std::collections::BTreeMap;
use std::fs;
use serde::{Serialize, Deserialize};
use messages::mqtt::Broker;

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    pub node_name: String,
    pub broker: Broker,
}

impl Config {
    pub fn new(config_path: &str) -> Config {
        let file_contents = fs::read_to_string(config_path).expect("Can't read config file");
        let config: Config = toml::from_str(file_contents.as_str()).expect("Can't parse config file");
        config
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Task {
    #[serde(default="default_task_image")]
    pub(crate) image: String,
    #[serde(default="default_task_cmds")]
    pub(crate) cmds: Option<Vec<String>>
}

fn default_task_image() -> String {
    String::from("alpine:latest")
}

fn default_task_cmds() -> Option<Vec<String>> {
    None
}

#[derive(Serialize, Deserialize, Clone)]
pub struct WorkerRequirements {
    pub(crate) min_no_cpus: u32,
}
#[derive(Serialize, Deserialize, Clone)]
pub struct JobDefinition {
    pub(crate) worker_requirements: WorkerRequirements,
    pub(crate) job: BTreeMap<String, Vec<Task>>
}

impl JobDefinition {
    pub fn new(config_path: &str) -> JobDefinition {
        let file_contents = fs::read_to_string(config_path).expect("Can't read config file");
        let job_definition: JobDefinition = toml::from_str(file_contents.as_str()).expect("Can't parse config file");
        job_definition
    }
    pub fn from_str(config_str: &str) -> JobDefinition {
        let job_definition: JobDefinition = toml::from_str(config_str).expect("Can't parse config file");
        job_definition
    }
}
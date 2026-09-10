use std::fs;
use messages::mqtt::Broker;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct CliConfig {
    pub client_name: String,
    pub broker: Broker,
}

impl CliConfig {
    pub fn from_file(config_path: &str) -> Self {
        let file_contents = fs::read_to_string(config_path).expect("Can't read config file");
        let conf: Self = toml::from_str(file_contents.as_str()).expect("Can't parse config file");
        conf
    }
}
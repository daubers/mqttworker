use std::fs;
use serde::{Serialize, Deserialize};


#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    pub node_name: String,
    pub broker: Broker,
}

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

impl Config {
    pub fn new(config_path: &str) -> Config {
        let file_contents = fs::read_to_string(config_path).expect("Can't read config file");
        let config: Config = toml::from_str(file_contents.as_str()).expect("Can't parse config file");
        config
    }
}
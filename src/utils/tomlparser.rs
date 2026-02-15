use toml;
use std::fs;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub socket: SocketStuff,
    pub routing: RoutingStuff,
}

#[derive(Deserialize)]
pub struct SocketStuff {
    pub address: String,
    pub port: u16,
}

#[derive(Deserialize)]
pub struct RoutingStuff {
    pub allow_path_params: Vec<String>,
}

pub fn server() -> Config {
    let conffile = fs::read_to_string("./config/server.toml").unwrap();
    let server: Config = toml::from_str(conffile.as_str()).unwrap();
    return server
}


use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs};

use crate::utils::compare_path;
use toml;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Endpoint {
    pub path: String,
    pub status: u16,
    pub body: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DB {
    pub path: String,
    pub data: String,
}

pub trait EndpointSearch {
    fn get_iter(&self, key: &str) -> Option<Endpoint>;
}

impl EndpointSearch for Vec<Endpoint> {
    fn get_iter(&self, key: &str) -> Option<Endpoint> {
        for endpoint in self {
            if compare_path(endpoint.path.to_string(), key.to_string()) {
                return Some(endpoint.clone());
            } else {
                continue;
            }
        }
        return None;
    }
}

#[derive(Serialize, Debug, Deserialize)]
pub struct Config {
    pub endpoints: HashMap<String, Vec<Endpoint>>,
    pub db: Option<Vec<DB>>,
}

impl Config {
    pub fn import(path: &str) -> Self {
        let config_toml = fs::read_to_string(path).unwrap();
        // Parsear el TOML
        let config: Config = toml::from_str(&config_toml).unwrap();
        return config;
    }
}

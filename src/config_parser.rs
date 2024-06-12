
use std::{collections::HashMap, fs};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use hteapot::{HttpMethod, HttpStatus};
use toml;
use crate::utils::compare_path;

#[derive(Serialize, Deserialize,Clone, Debug)]
pub struct Endpoint {
  pub path: String,
  pub status: u16,
  pub body: String,
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
  pub endpoints: HashMap<String,Vec<Endpoint>>
}

 
impl Config {
    pub fn new() -> Self {
      let mut endpoints = HashMap::new();
      let e1 = Endpoint {
        path: "/test".to_string(),
        status: 200,
        body: "Hello world".to_string()
      };
      let e2 = Endpoint {
        path: "/".to_string(),
        status: 201,
        body: "abc".to_string()
      };      
      let e3 = Endpoint {
        path: "/error".to_string(),
        status: 404,
        body: "error".to_string()
      };

      endpoints.insert(HttpMethod::GET.to_str().to_string(), vec![e1,e2,e3]);
      Config {
        endpoints: endpoints
      }
  }


  pub fn to_str(&self) -> String {
    toml::to_string(self).unwrap().to_string()
  }

  pub fn import(path: &str) -> Self {
    let config_toml = fs::read_to_string(path).unwrap();
    // Parsear el TOML
    let config: Config = toml::from_str(&config_toml).unwrap();
    return config;

  }

}

use std::{collections::HashMap, fs::File, io::BufReader};
use serde::{Deserialize, Serialize};
use serde_json::{Result, Value};

use crate::hteapot::HttpMethod;


#[derive(Serialize, Deserialize)]
#[derive(Clone)]
pub struct response {
  pub status: u16,
  pub body: Value,
}
pub type responseMap = HashMap<String, response>;

pub type configMap = HashMap<HttpMethod, responseMap>;

pub trait config {
  fn new() -> Self;
  fn import(path: &str) -> Self;
}

impl config for configMap{
    fn new() -> Self {
      return HashMap::new();
  }

  fn import(path: &str) -> Self {
    let file = File::open(path).unwrap();
    let reader = BufReader::new(file);
    let raw: Value = serde_json::from_reader(reader).unwrap();
    let mut data = HashMap::new();
    for (key, value) in raw.as_object().unwrap() {
      let mut responses = HashMap::new();
      for element in value.as_array().unwrap() {
        let element = element.as_object().unwrap();
        let path = element.keys().next().unwrap();
        let response = element.get(path).unwrap();
        let response = response {
          status: response["status"].as_u64().unwrap() as u16,
          body: response["body"].clone(),
        };
        responses.insert(path.clone(), response);
      }
      let method = HttpMethod::from_str(key.to_uppercase().as_str());
      data.insert(method, responses);
    }
    return data;
  }

}
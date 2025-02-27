use std::collections::HashMap;

use serde_json::Value;

// EXAMPLE JSON DB
// [
//  {
//      name: "Alberto",
//      surname: "Ruiz",
//      age: 25,
//      admin: True,
//
//  },
//  {
//      name: "Eithne",
//      surname: "Flor",
//      age: 21,
//      admin: False
//  },
//  {
//      name: "Alberto",
//      surname: "Ruiz",
//      age: 25,
//      admin: True
//  },
// ]

pub struct DbHandle {
    pub root_path: String,
    db_data: Value,
}

impl DbHandle {
    pub fn new(root_path: String, json: String) -> Result<Self, &'static str> {
        let db_data = serde_json::from_str(json.as_str());
        if db_data.is_err() {
            println!("{:?}", db_data.err());
            return Err("Invalid db json");
        }
        let db_data: Value = db_data.unwrap();

        Ok(DbHandle { root_path, db_data })
    }

    fn post(
        &mut self,
        path: String,
        _args: HashMap<String, String>,
        body: Option<Value>,
    ) -> Option<String> {
        let pointer = self.db_data.pointer_mut(&path)?;
        if pointer.is_array() {
            let list = pointer.as_array_mut()?;
            let r = list.push(body?);
            let _ = serde_json::to_string(&r.clone());
        } else {
            let body_c = body.clone()?;
            let body_obj = body_c.as_object()?;
            for (k, v) in body_obj.clone() {
                pointer[k] = v.clone();
            }
        }
        let result = serde_json::to_string(&pointer);
        match result {
            Ok(r) => {
                if r == "null" {
                    None
                } else {
                    Some(r)
                }
            }
            Err(_) => None,
        }
    }

    fn get(&self, path: String, args: HashMap<String, String>) -> Option<String> {
        let mut pointer = self.db_data.pointer(&path)?.clone();
        if pointer.is_array() {
            let mut array: Vec<Value> = pointer.as_array().unwrap().clone();
            for (k, v) in args {
                let k: &str = k.as_str().as_ref();
                array = array
                    .into_iter()
                    .filter(|i| i[k].to_string() == v)
                    .collect::<Vec<Value>>();
            }
            pointer = Value::Array(array);
        }
        let result = serde_json::to_string(&pointer);
        match result {
            Ok(r) => {
                if r == "null" {
                    None
                } else {
                    Some(r)
                }
            }
            Err(_) => None,
        }
    }

    pub fn is_match(&self, path: &String) -> bool {
        path.starts_with(self.root_path.as_str())
    }

    pub fn process(
        &mut self,
        method: &str,
        path: String,
        args: HashMap<String, String>,
        body: String,
    ) -> Option<String> {
        let mut path = path;
        let root_path = if self.root_path.ends_with('/') {
            let mut chars = self.root_path.chars();
            chars.next_back();
            chars.as_str()
        } else {
            self.root_path.as_str()
        };
        if path.starts_with(root_path) {
            path = path.strip_prefix(root_path).unwrap().to_string();
        } else {
            return None;
        }
        let path = if path.ends_with('/') {
            let mut path = path.clone();
            path.pop();
            path
        } else {
            path
        };
        let body = serde_json::from_str::<Value>(&body);
        let body = if body.is_err() {
            None
        } else {
            Some(body.unwrap())
        };
        match method {
            "GET" => self.get(path, args),
            "POST" => self.post(path, args, body),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_main_valid_json() {
        let root_path = String::from("/path/to/db");
        let json_data = json!({"key1": "value1", "key2": {"subkey": "value2"}}).to_string();

        let db_handle = DbHandle::new(root_path.clone(), json_data.clone()).unwrap();

        assert_eq!(db_handle.root_path, root_path);
        assert_eq!(
            db_handle.db_data,
            serde_json::from_str::<Value>(&json_data).unwrap()
        );
    }

    #[test]
    fn test_main_invalid_json() {
        let root_path = String::from("/path/to/db");
        let invalid_json_data = String::from("{invalid_json}");

        let result = DbHandle::new(root_path, invalid_json_data);
        assert!(result.is_err());
        assert_eq!(result.err(), Some("Invalid db json"));
    }

    #[test]
    fn test_get_valid_path() {
        let root_path = String::from("/path/to/db");
        let json_data = json!({"key1": "value1", "key2": {"subkey": "value2"}}).to_string();
        let db_handle = DbHandle::new(root_path.clone(), json_data).unwrap();

        let result = db_handle.get(String::from("key2/subkey"), HashMap::new());
        assert_eq!(result, Some(String::from("\"value2\""))); // Serde JSON añade comillas a las cadenas
    }

    #[test]
    fn test_get_invalid_path() {
        let root_path = String::from("/path/to/db");
        let json_data = json!({"key1": "value1", "key2": {"subkey": "value2"}}).to_string();
        let db_handle = DbHandle::new(root_path.clone(), json_data).unwrap();

        let result = db_handle.get(String::from("key2/nonexistent"), HashMap::new());
        assert_eq!(result, None); // No existe la clave
    }

    #[test]
    fn test_get_empty_path() {
        let root_path = String::from("/path/to/db");
        let json_data = json!({"key1": "value1", "key2": {"subkey": "value2"}}).to_string();
        let db_handle = DbHandle::new(root_path.clone(), json_data).unwrap();

        let result = db_handle.get(String::from(""), HashMap::new());
        assert_eq!(result, None); // Ruta vacía no debe devolver nada
    }

    #[test]
    fn test_get_path_with_multiple_elements() {
        let root_path = String::from("/path/to/db");
        let json_data =
            json!({"key1": "value1", "key2": {"subkey": {"deepkey": "deepvalue"}}}).to_string();
        let db_handle = DbHandle::new(root_path.clone(), json_data).unwrap();

        let result = db_handle.get(String::from("key2/subkey/deepkey"), HashMap::new());
        assert_eq!(result, Some(String::from("\"deepvalue\""))); // Verifica el valor profundo
    }
}

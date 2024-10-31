use serde_json::{json, Value};

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

    fn get(&self, path: String) -> Option<String> {
        let path_elements = path.split("/");
        let mut pointer = self.db_data.clone();

        for element in path_elements.into_iter() {
            if element == "" {
                continue;
            }
            if pointer.is_array() {
                let pointer_array = pointer.as_array().unwrap();
                let rp = pointer_array
                    .iter()
                    .find(|v| v["id"].to_string() == element);
                if rp.is_some() {
                    pointer = rp.unwrap().clone();
                } else {
                    let index = element.parse::<usize>().unwrap();
                    pointer = pointer_array[index].clone();
                }
            } else {
                pointer = pointer[element].clone();
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

    pub fn is_match(&self, path: &String) -> bool {
        path.starts_with(self.root_path.as_str())
    }

    pub fn process(&self, method: &str, path: String) -> Option<String> {
        let mut path = path;
        if path.starts_with(self.root_path.as_str()) {
            path = path
                .strip_prefix(self.root_path.as_str())
                .unwrap()
                .to_string();
        } else {
            return None;
        }
        match method {
            "GET" => self.get(path),
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

        let result = db_handle.get(String::from("key2/subkey"));
        assert_eq!(result, Some(String::from("\"value2\""))); // Serde JSON añade comillas a las cadenas
    }

    #[test]
    fn test_get_invalid_path() {
        let root_path = String::from("/path/to/db");
        let json_data = json!({"key1": "value1", "key2": {"subkey": "value2"}}).to_string();
        let db_handle = DbHandle::new(root_path.clone(), json_data).unwrap();

        let result = db_handle.get(String::from("key2/nonexistent"));
        assert_eq!(result, None); // No existe la clave
    }

    #[test]
    fn test_get_empty_path() {
        let root_path = String::from("/path/to/db");
        let json_data = json!({"key1": "value1", "key2": {"subkey": "value2"}}).to_string();
        let db_handle = DbHandle::new(root_path.clone(), json_data).unwrap();

        let result = db_handle.get(String::from(""));
        assert_eq!(result, None); // Ruta vacía no debe devolver nada
    }

    #[test]
    fn test_get_path_with_multiple_elements() {
        let root_path = String::from("/path/to/db");
        let json_data =
            json!({"key1": "value1", "key2": {"subkey": {"deepkey": "deepvalue"}}}).to_string();
        let db_handle = DbHandle::new(root_path.clone(), json_data).unwrap();

        let result = db_handle.get(String::from("key2/subkey/deepkey"));
        assert_eq!(result, Some(String::from("\"deepvalue\""))); // Verifica el valor profundo
    }
}

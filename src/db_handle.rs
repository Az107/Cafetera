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

    fn _find(&mut self, path: String) -> Option<&mut Value> {
        let path_elements = path.split("/");
        let mut pointer = &mut self.db_data;

        for element in path_elements.into_iter() {
            if element == "" {
                continue;
            }
            if pointer.is_array() {
                let array_pointer = pointer.as_array_mut().unwrap();
                let array_pointer_clone = array_pointer.clone();
                let index = if let Ok(idx) = element.parse::<usize>() {
                    Some(idx)
                } else {
                    // Busca el índice del elemento con el "id" específico.
                    array_pointer_clone
                        .iter()
                        .position(|v| v["id"].to_string() == element)
                };
                if index.is_none() {
                    return None;
                }
                let index = index.unwrap();
                if index >= array_pointer_clone.len() {
                    return None;
                }
                pointer = &mut array_pointer[index];
            } else {
                pointer = &mut pointer[element];
            }
        }
        return Some(pointer);
    }

    fn filter(data: Value, args: HashMap<String, String>) -> Option<Value> {
        let mut result = data.clone();
        if args.len() == 0 {
            return Some(data);
        }
        if data.is_array() {
            let mut array: Vec<Value> = data.as_array().unwrap().clone();
            for (k, v) in args {
                let k: &str = k.as_str().as_ref();
                array = array
                    .into_iter()
                    .filter(|i| i[k].to_string() == v)
                    .collect::<Vec<Value>>();
            }
            result = Value::Array(array);
        } else {
            return Some(data);
        }
        return Some(result);
    }

    fn get(&mut self, path: String, args: HashMap<String, String>) -> Option<Value> {
        let result = self._find(path);
        if result.is_none() {
            return None;
        }
        let result = result.unwrap().clone();
        let result = Self::filter(result, args);
        return result;
    }

    fn post(&mut self, path: String, args: HashMap<String, String>, body: String) -> Option<Value> {
        let pointer = self._find(path);
        if pointer.is_none() {
            return None;
        }
        let pointer = pointer.unwrap();

        if pointer.is_array() {
            let new_object = serde_json::from_str(body.as_str());
            if new_object.is_err() {
                return None;
            }
            let new_object = new_object.unwrap();
            pointer.as_array_mut().unwrap().push(new_object);
        }
        return Some(pointer.clone());
    }

    fn delete(
        &mut self,
        path: String,
        args: HashMap<String, String>,
        body: String,
    ) -> Option<Value> {
        let result = if args.len() != 0 {
            let pointer = self._find(path);
            if pointer.is_none() {
                return None;
            }
            let pointer = pointer.unwrap();
            if pointer.is_array() {
                let array_pointer = pointer.as_array_mut().unwrap();
                array_pointer.retain(|x| {
                    let mut remove = false;
                    for (k, v) in args.iter() {
                        let k: &str = k.as_str().as_ref();
                        if x[k].to_string() == *v {
                            remove = true;
                        } else {
                            remove = false;
                        }
                    }

                    return !remove;
                });
            }

            Some(pointer.clone())
        } else {
            let target = path.split('/').last().unwrap_or("");
            let parent = path.strip_suffix(target).unwrap_or("/").to_string();
            let pointer = self._find(parent);
            if pointer.is_none() {
                return None;
            }
            let pointer = pointer.unwrap();
            if pointer.is_object() {
                let obj = pointer.as_object_mut().unwrap();
                obj.remove(target);
            } else if pointer.is_array() && target.parse::<usize>().is_ok() {
                let arr = pointer.as_array_mut().unwrap();
                let index = target.parse::<usize>().unwrap();
                arr.remove(index);
            }
            Some(pointer.clone())
        };

        return result;
    }

    pub fn is_match(&self, path: &String) -> bool {
        path.starts_with(self.root_path.as_str())
    }

    fn friendlify(data: Option<Value>) -> Option<String> {
        if data.is_none() {
            return None;
        }
        let data = data.unwrap();

        let result = serde_json::to_string(&data.clone());
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
        let response = match method {
            "GET" => self.get(path, args),
            "POST" => self.post(path, args, body),
            "DELETE" => self.delete(path, args, body),
            _ => None,
        };
        return Self::friendlify(response);
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
        let mut db_handle = DbHandle::new(root_path.clone(), json_data).unwrap();

        let result = db_handle.get(String::from("key2/subkey"), HashMap::new());
        let result = DbHandle::friendlify(result);
        assert_eq!(result, Some(String::from("\"value2\""))); // Serde JSON añade comillas a las cadenas
    }

    #[test]
    fn test_get_invalid_path() {
        let root_path = String::from("/path/to/db");
        let json_data = json!({"key1": "value1", "key2": {"subkey": "value2"}}).to_string();
        let mut db_handle = DbHandle::new(root_path.clone(), json_data).unwrap();

        let result = db_handle.get(String::from("key2/nonexistent"), HashMap::new());
        assert_eq!(result, None); // No existe la clave
    }

    // #[test]
    // fn test_get_empty_path() {
    //     let root_path = String::from("/path/to/db");
    //     let json_data = json!({"key1": "value1", "key2": {"subkey": "value2"}}).to_string();
    //     let db_handle = DbHandle::new(root_path.clone(), json_data).unwrap();

    //     let result = db_handle.get(String::from(""), HashMap::new());
    //     assert_eq!(result, None); // Ruta vacía no debe devolver nada
    // }

    #[test]
    fn test_get_path_with_multiple_elements() {
        let root_path = String::from("/path/to/db");
        let json_data =
            json!({"key1": "value1", "key2": {"subkey": {"deepkey": "deepvalue"}}}).to_string();
        let mut db_handle = DbHandle::new(root_path.clone(), json_data).unwrap();

        let result = db_handle.get(String::from("key2/subkey/deepkey"), HashMap::new());
        let result = DbHandle::friendlify(result);
        assert_eq!(result, Some(String::from("\"deepvalue\""))); // Verifica el valor profundo
    }
}

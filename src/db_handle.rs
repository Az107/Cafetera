use hteapot::HttpStatus;
use serde_json::Value;
use std::collections::HashMap;
// DB Module to manage quick mock of dbs
// this allow basic CRUD whit a mock DB in config
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

pub struct HttpErr {
    pub status: HttpStatus,
    pub text: &'static str,
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
    fn split_path(path: &str) -> Option<(&str, &str)> {
        let mut parts = path.rsplitn(2, '/');
        let attribute = parts.next()?;
        let parent = parts.next().unwrap_or("");
        Some((parent, attribute))
    }

    fn delete(&mut self, path: String, _args: HashMap<String, String>) -> Result<String, HttpErr> {
        let _ = self.db_data.pointer(&path).ok_or(HttpErr {
            status: HttpStatus::NotFound,
            text: "Invalud path",
        });
        let (parent, attr) = Self::split_path(&path).ok_or(HttpErr {
            status: HttpStatus::BadRequest,
            text: "Can't remove all the db",
        })?;
        let pointer = self.db_data.pointer_mut(&parent).ok_or(HttpErr {
            status: HttpStatus::NotFound,
            text: "Parent not found",
        })?;
        if pointer.is_array() {
            let index = attr.parse::<usize>().map_err(|_| HttpErr {
                status: HttpStatus::BadRequest,
                text: "Invalid path",
            })?;
            pointer
                .as_array_mut()
                .ok_or(HttpErr {
                    status: HttpStatus::BadRequest,
                    text: "Invalid Path",
                })?
                .remove(index);
        } else if pointer.is_object() {
            pointer
                .as_object_mut()
                .ok_or(HttpErr {
                    status: HttpStatus::BadRequest,
                    text: "Invalid Path",
                })?
                .remove(attr);
        }
        let result = pointer.to_string();
        return Ok(result);
    }

    fn patch(
        &mut self,
        path: String,
        _args: HashMap<String, String>,
        body: Option<Value>,
    ) -> Result<String, HttpErr> {
        let pointer = self.db_data.pointer_mut(&path).ok_or(HttpErr {
            status: HttpStatus::NotFound,
            text: "Path Not Found",
        })?;
        if pointer.is_array() {
            return Err(HttpErr {
                status: HttpStatus::BadRequest,
                text: "Invalid Path",
            });
        } else {
            let body_c = body.clone().ok_or(HttpErr {
                status: HttpStatus::BadRequest,
                text: "Invalid Body",
            })?;
            let body_obj = body_c.as_object().ok_or(HttpErr {
                status: HttpStatus::BadRequest,
                text: "Invalid Body",
            })?;
            for (k, v) in body_obj.clone() {
                if pointer.get(k.clone()).is_none() {
                    continue;
                };
                pointer[k] = v.clone();
            }
        }
        let result = serde_json::to_string(&pointer);
        match result {
            Ok(r) => {
                if r == "null" {
                    Err(HttpErr {
                        status: HttpStatus::NotFound,
                        text: "Not Found",
                    })
                } else {
                    Ok(r)
                }
            }
            Err(_) => Err(HttpErr {
                status: HttpStatus::InternalServerError,
                text: "Error parsing result",
            }),
        }
    }

    fn post(
        &mut self,
        path: String,
        _args: HashMap<String, String>,
        body: Option<Value>,
    ) -> Result<String, HttpErr> {
        let pointer = self.db_data.pointer_mut(&path).ok_or(HttpErr {
            status: HttpStatus::BadRequest,
            text: "Invalid Path",
        })?;
        if pointer.is_array() {
            let list = pointer.as_array_mut().ok_or(HttpErr {
                status: HttpStatus::BadRequest,
                text: "Invalid Path",
            })?;
            let r = list.push(body.ok_or(HttpErr {
                status: HttpStatus::BadRequest,
                text: "Invalid Body",
            })?);
            let _ = serde_json::to_string(&r.clone());
        } else {
            let body_c = body.clone().ok_or(HttpErr {
                status: HttpStatus::BadRequest,
                text: "Invalid Body",
            })?;
            let body_obj = body_c.as_object().ok_or(HttpErr {
                status: HttpStatus::BadRequest,
                text: "Invalid Body",
            })?;
            for (k, v) in body_obj.clone() {
                pointer[k] = v.clone();
            }
        }
        let result = serde_json::to_string(&pointer);
        match result {
            Ok(r) => {
                if r == "null" {
                    Err(HttpErr {
                        status: HttpStatus::NotFound,
                        text: "Not Found",
                    })
                } else {
                    Ok(r)
                }
            }
            Err(_) => Err(HttpErr {
                status: HttpStatus::InternalServerError,
                text: "Error parsing result",
            }),
        }
    }

    fn get(&self, path: String, args: HashMap<String, String>) -> Result<String, HttpErr> {
        let mut pointer = self
            .db_data
            .pointer(&path)
            .ok_or(HttpErr {
                status: HttpStatus::BadRequest,
                text: "Invalid Path",
            })?
            .clone();
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
                    Err(HttpErr {
                        status: HttpStatus::NotFound,
                        text: "Not Found",
                    })
                } else {
                    Ok(r)
                }
            }
            Err(_) => Err(HttpErr {
                status: HttpStatus::InternalServerError,
                text: "Error parsing result",
            }),
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
    ) -> Result<String, HttpErr> {
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
            return Err(HttpErr {
                status: HttpStatus::BadRequest,
                text: "Invalid Path",
            });
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
            "PATCH" => self.patch(path, args, body),
            "DELETE" => self.delete(path, args),
            _ => Err(HttpErr {
                status: HttpStatus::MethodNotAllowed,
                text: "Method Not Allowed",
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_dbhandle_new_valid_json() {
        let json_data = json!({"key": "value"}).to_string();
        let db = DbHandle::new("/test".to_string(), json_data);
        assert!(db.is_ok());
    }

    #[test]
    fn test_dbhandle_new_invalid_json() {
        let json_data = "invalid_json".to_string();
        let db = DbHandle::new("/test".to_string(), json_data);
        assert!(db.is_err());
    }

    #[test]
    fn test_delete_valid_key() {
        let json_data = json!({"parent": {"child": "value"}}).to_string();
        let mut db: DbHandle = DbHandle::new("/test".to_string(), json_data).unwrap();
        let result = db.delete("/parent/child".to_string(), HashMap::new());
        assert!(result.is_ok())
        //assert_eq!(result.unwrap(), "{}");
    }

    #[test]
    fn test_delete_invalid_key() {
        let json_data = json!({"parent": {"child": "value"}}).to_string();
        let mut db: DbHandle = DbHandle::new("/test".to_string(), json_data).unwrap();
        let result = db.delete("/paren/non_existent".to_string(), HashMap::new());
        assert!(result.is_err());
    }

    #[test]
    fn test_patch_valid_key() {
        let json_data = json!({"key": {"subkey": "old_value"}}).to_string();
        let mut db: DbHandle = DbHandle::new("/test".to_string(), json_data).unwrap();
        let patch_body = json!({"subkey": "new_value"});
        let result = db.patch("/key".to_string(), HashMap::new(), Some(patch_body));
        assert!(result.is_ok());
        //assert_eq!(result.unwrap(), "{\"subkey\":\"new_value\"}");
    }

    #[test]
    fn test_post_valid_key() {
        let json_data = json!({"list": []}).to_string();
        let mut db = DbHandle::new("/test".to_string(), json_data).unwrap();
        let post_body = json!("new_item");
        let result = db.post("/list".to_string(), HashMap::new(), Some(post_body));
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_valid_key() {
        let json_data = json!({"key": "value"}).to_string();
        let db = DbHandle::new("/test".to_string(), json_data).unwrap();
        let result = db.get("/key".to_string(), HashMap::new());
        assert!(result.is_ok());
        //assert_eq!(result.unwrap(), "\"value\"");
    }

    #[test]
    fn test_get_invalid_key() {
        let json_data = json!({"key": "value"}).to_string();
        let db = DbHandle::new("/test".to_string(), json_data).unwrap();
        let result = db.get("/non_existent".to_string(), HashMap::new());
        assert!(result.is_err());
    }
}

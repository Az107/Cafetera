mod config_parser;
mod db_handle;
mod utils;

use std::sync::{Arc, Mutex};

use config_parser::{Config, EndpointSearch};
use db_handle::DbHandle;
use hteapot::{headers, Hteapot, HttpMethod, HttpResponse, HttpStatus};
use serde_json::Value;
use utils::{clean_arg, print_args, SimpleRNG};

use crate::utils::now;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
    let args = std::env::args().collect::<Vec<String>>();
    if let Some(two) = args.get(2) {
        if two == "-v" || two == "--version" {
            println!("Cafetera {}", VERSION);
            // println!("Hteapot {}", hteapot::VERSION);
            return;
        }
    }
    if args.len() < 3 {
        println!("Usage: {} <port> <config>", args[0]);
        return;
    }
    let addr: String = String::from("0.0.0.0");
    let port: u16 = args[1].clone().parse().unwrap_or(8080);
    let config = Config::import(&args[2]);
    let silent = args
        .get(3)
        .is_some()
        .then(|| args.get(3).unwrap().eq("-s"))
        .is_some();
    let mut dbs: Vec<db_handle::DbHandle> = Vec::new();
    for method in config.endpoints.keys() {
        for endpoint in config.endpoints[method].iter() {
            println!("Loaded {} {}", method, endpoint.path)
        }
    }
    if config.db.is_some() {
        let config_db = config.db.unwrap().clone();
        for db in config_db {
            let dbh = db_handle::DbHandle::new(db.path, db.data);
            if dbh.is_err() {
                println!("Error loading db: {:?}", dbh.err());
                continue;
            }
            let dbh = dbh.unwrap();
            println!("Loaded {} as db", dbh.root_path);
            dbs.push(dbh);
        }
    }
    let dbs: Arc<Mutex<Vec<DbHandle>>> = Arc::new(Mutex::new(dbs));
    let dbsc = dbs.clone();
    let teapot = Hteapot::new(&addr, port);
    println!("Listening on http://{}:{}", addr, port);
    teapot.listen(move|req| {
            let body_text = req.text().unwrap_or(String::new());

            if !silent {
                let headers: String = req.headers.iter()
                    .map(|(k, v)| format!("- {}: {}", k, v))
                    .collect::<Vec<String>>()
                    .join("\n");

                let output = format!(
                    "{} {}{}\n{}\n\n{}",
                    req.method.to_str(),
                    req.path,
                    print_args(&req.args),
                    headers,
                    body_text
                );

                println!("{}", output);
            }
            let star = "*";
            let origin = req.headers.get_owned("Origin").unwrap_or(star.to_string());
            let request_headers = req.headers.get_owned("access-control-request-headers").unwrap_or(star.to_string());
            let mut headers = headers!("Allow" => "GET, POST, OPTIONS, HEAD", "Access-Control-Allow-Origin" => &origin, "Access-Control-Allow-Headers" => &request_headers ).unwrap();

            if req.method == HttpMethod::OPTIONS {

                return HttpResponse::new(HttpStatus::NoContent, "",  Some(headers));
            }



            {
                let mut dbs = dbsc.lock().unwrap();
                let dbh = dbs.iter_mut().find(|dbh| dbh.is_match(&req.path));
                if dbh.is_some() {
                    let dbh = dbh.unwrap();
                    let result = dbh.process(req.method.to_str(), req.path, req.args,body_text);
                    headers.insert("Content-Type", "application/json");
                    return match result {
                        Ok(r) => HttpResponse::new(HttpStatus::OK, r, Some(headers)),
                        Err(err) => HttpResponse::new(err.status, err.text ,None )
                        }
                }
            }

            let response = config.endpoints.get(&req.method.to_str().to_string());
            match response {
                Some(response) => {
                    let config_item = response.get_iter(&req.path);
                    match config_item {
                        Some(endpoint) => {
                            let status = HttpStatus::from_u16(endpoint.status).unwrap_or(HttpStatus::OK);
                            let mut body = endpoint.body.to_string()
                            .replace("{{path}}", &req.path)
                            .replace("{{body}}", &body_text)
                            .replace("{{rand}}", SimpleRNG::new().next_range(0, 100).to_string().as_str())
                            .replace("{{now}}", &now().to_string());
                            for (key, value) in &req.args {
                                let _body = body.clone();
                                body = _body.replace(&format!("{{{{arg.{key}}}}}", key=key),  clean_arg(value.to_string()).as_str());
                            }
                            let path_args = utils::get_path_args(req.path.clone(), endpoint.path.clone());
                            if path_args.is_some() {
                                for (key, value) in path_args.unwrap() {
                                    let _body = body.clone();
                                    body = _body.replace(&format!("{{{{{key}}}}}", key=key), &value);
                                }
                            }
                            if  serde_json::from_str::<Value>(&body).is_ok() {
                                headers.insert("Content-Type", "application/json");
                            }
                            return HttpResponse::new(status, &body,Some(headers) );
                        }
                        None => {
                            return HttpResponse::new(HttpStatus::NotFound, "Not Found", None);
                        }
                    }
                }
                None => {
                    return HttpResponse::new(HttpStatus::NotFound, "Method Not Found", None);
                }
            }

        });
}

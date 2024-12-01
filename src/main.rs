mod config_parser;
mod db_handle;
mod utils;

use config_parser::{Config, EndpointSearch};
use hteapot::headers;
use hteapot::HttpMethod;
use hteapot::{Hteapot, HttpStatus};
use utils::clean_arg;
use utils::SimpleRNG;

// section MAIN

fn main() {
    let args = std::env::args().collect::<Vec<String>>();
    if args.len() != 3 {
        println!("Usage: {} <port> <config>", args[0]);
        return;
    }
    let addr: String = String::from("0.0.0.0");
    let port: u16 = args[1].clone().parse().unwrap_or(8080);
    let config = Config::import(&args[2]);
    let mut dbs: Vec<db_handle::DbHandle> = Vec::new();
    for method in config.endpoints.keys() {
        for endpoint in config.endpoints[method].iter() {
            println!("Loaded {} {}", method, endpoint.path)
        }
    }
    for db in config.db {
        let dbh = db_handle::DbHandle::new(db.path, db.data);
        if dbh.is_err() {
            println!("Error loading db: {:?}", dbh.err());
            continue;
        }
        let dbh = dbh.unwrap();
        println!("Loaded {} as db", dbh.root_path);
        dbs.push(dbh);
    }
    let teapot = Hteapot::new(&addr, port);
    println!("Listening on http://{}:{}", addr, port);
    teapot.listen(move|req| {
            println!("{} {}", req.method.to_str(), req.path);
            println!("{:?}", req.headers);
            println!("{}", req.body);
            println!();
            if req.method == HttpMethod::OPTIONS {
                let star = &"*".to_string();
                let origin = req.headers.get("Origin").unwrap_or(star);
                return Hteapot::response_maker(HttpStatus::NoContent, "", headers!("Allow" => "GET, POST, OPTIONS, HEAD", "Access-Control-Allow-Origin" => origin, "Access-Control-Allow-Headers" => "Content-Type, Authorization" ));
            }



            let dbh = dbs.iter().find(|&dbh| dbh.is_match(&req.path));
            if dbh.is_some() {
                let dbh = dbh.unwrap();
                let result = dbh.process(req.method.to_str(), req.path, req.args);
                return match result {
                    Some(r) => Hteapot::response_maker(HttpStatus::OK, r,None ),
                    None => Hteapot::response_maker(HttpStatus::NotFound, "DB query not found" ,None )
                    }
            }

            let response = config.endpoints.get(&req.method.to_str().to_string());
            match response {
                Some(response) => {
                    let config_item = response.get_iter(&req.path);
                    match config_item {
                        Some(endpoint) => {
                            let status = HttpStatus::from_u16(endpoint.status);
                            let mut body = endpoint.body.to_string()
                            .replace("{{path}}", &req.path)
                            .replace("{{body}}", &req.body)
                            .replace("{{rand}}", SimpleRNG::new().next_range(0, 100).to_string().as_str());
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
                            return Hteapot::response_maker(status, &body,headers!("Allow" => "GET, POST, OPTIONS, HEAD", "Access-Control-Allow-Origin" => "*", "Access-Control-Allow-Headers" => "Content-Type, Authorization" ) );
                        }
                        None => {
                            return Hteapot::response_maker(HttpStatus::NotFound, "Not Found", None);
                        }
                    }
                }
                None => {
                    return Hteapot::response_maker(HttpStatus::NotFound, "Method Not Found", None);
                }
            }

        });
}

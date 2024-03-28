mod hteapot;
mod config_parser;
use std::env;

use hteapot::HteaPot;
use config_parser::{configMap, config };
use crate::hteapot::{HttpMethod, HttpStatus};


const DEFAULT_PORT: &str = "7878";

fn main() {
        let args = std::env::args().collect::<Vec<String>>();
        if args.len() != 3 {
            println!("Usage: {} <server> <port>", args[0]);
            return;
        }
        let addr: String = String::from("0.0.0.0");
        let port: u16 = args[1].clone().parse().unwrap_or(8080);
        let config = config_parser::configMap::import(&args[2]);
        let teapot = HteaPot::new(&addr, port);
        println!("Listening on http://{}:{}", addr, port);
        teapot.listen(|req| {
            println!("{} {}", req.method.to_str(), req.path);
            println!("{:?}", req.headers);
            println!("{}", req.body);
            println!();
            let response = config.get(&req.method);
            match response {
                Some(response) => {
                    let response = response.get(&req.path);
                    match response {
                        Some(response) => {
                            let status = HttpStatus::from_u16(response.status);
                            let body = &response.body.to_string().replace("{{path}}", &req.path).replace("{{body}}", &req.body);
                            return HteaPot::response_maker(status, body );
                        }
                        None => {
                            return HteaPot::response_maker(HttpStatus::NotFound, "Not Found");
                        }
                    }
                }
                None => {
                    return HteaPot::response_maker(HttpStatus::NotFound, "Method Not Found");
                }
            } 

        });

}

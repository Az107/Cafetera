use std::{
    collections::HashMap,
    time::{SystemTime, UNIX_EPOCH},
};

pub struct SimpleRNG {
    state: u64,
}

impl SimpleRNG {
    pub fn new() -> Self {
        let seed = std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        SimpleRNG { state: seed }
    }

    pub fn next(&mut self) -> u64 {
        self.state = self.state.wrapping_mul(6364136223846793061).wrapping_add(1);
        self.state >> 16
    }

    pub fn next_range(&mut self, min: u64, max: u64) -> u64 {
        let scaled_range = max - min;
        let scaled_random = self.next() % scaled_range;
        min + scaled_random
    }
}

pub fn compare_path(path: String, path2: String) -> bool {
    let parts = path.split("/");
    let parts2 = path2.split("/");
    if parts.clone().count() != parts2.clone().count() {
        return false;
    }
    for (part, part2) in parts.zip(parts2) {
        if part == part2 {
            continue;
        } else if part.starts_with("{") && part.ends_with("}") {
            continue;
        } else {
            return false;
        }
    }

    return true;
}

pub fn clean_arg(arg_value: String) -> String {
    let url_decoding_map: HashMap<&str, char> = vec![
        ("%20", ' '),
        ("%21", '!'),
        ("%22", '"'),
        ("%23", '#'),
        ("%24", '$'),
        ("%25", '%'),
        ("%26", '&'),
        ("%27", '\''),
        ("%28", '('),
        ("%29", ')'),
        ("%2A", '*'),
        ("%2B", '+'),
        ("%2C", ','),
        ("%2D", '-'),
        ("%2E", '.'),
        ("%2F", '/'),
        ("%3A", ':'),
        ("%3B", ';'),
        ("%3C", '<'),
        ("%3D", '='),
        ("%3E", '>'),
        ("%3F", '?'),
        ("%40", '@'),
        ("%5B", '['),
        ("%5C", '\\'),
        ("%5D", ']'),
        ("%5E", '^'),
        ("%5F", '_'),
        ("%60", '`'),
        ("%7B", '{'),
        ("%7C", '|'),
        ("%7D", '}'),
        ("%7E", '~'),
    ]
    .into_iter()
    .collect();
    let mut decoded = arg_value.clone();
    for item in url_decoding_map.keys() {
        decoded = decoded.replace(
            item,
            url_decoding_map.get(item).unwrap().to_string().as_str(),
        );
    }
    return decoded;
}

pub fn get_path_args(path: String, path2: String) -> Option<HashMap<String, String>> {
    let parts = path.split("/");
    let parts2 = path2.split("/");
    let mut params = HashMap::new();
    for (part, part2) in parts.zip(parts2) {
        if part == part2 {
            continue;
        } else if part2.starts_with("{") && part2.ends_with("}") {
            let key = part2
                .trim_start_matches("{{")
                .trim_end_matches("}}")
                .to_string();
            params.insert(key, part.to_string());
            continue;
        } else {
            return None;
        }
    }

    return Some(params);
}

pub fn print_args(args: &HashMap<String, String>) -> String {
    let mut result = String::new();
    result.push('?');
    for (k, v) in args {
        result.push_str(format!("{}={}&", k, v).as_str());
    }
    result.pop();
    return result;
}

pub fn now() -> u64 {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("time should go forward");
    since_the_epoch.as_secs()
}

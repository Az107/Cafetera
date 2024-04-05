use std::{collections::HashMap, path::Path};

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

    pub fn new_with_seed(seed: u64) -> Self {
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

pub fn get_path_args(path: String, path2: String) -> Option<HashMap<String,String>> {
  let parts = path.split("/");
  let parts2 = path2.split("/");
  let mut params = HashMap::new();
  for (part, part2) in parts.zip(parts2) {
    if part == part2 {
      continue;
    } else if part2.starts_with("{") && part2.ends_with("}") {
      let key = part2.trim_start_matches("{{").trim_end_matches("}}").to_string();
      params.insert(key, part.to_string());
      continue;
    } else {
      return None;
    }
  }

  return Some(params);
}
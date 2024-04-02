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
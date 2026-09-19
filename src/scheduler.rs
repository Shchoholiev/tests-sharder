use std::collections::BinaryHeap;

#[derive(PartialEq, Eq)]
pub struct Test {
  pub id: String,
  pub duration_ms: u32,
}

impl Ord for Test {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.duration_ms.cmp(&other.duration_ms)
    }
}

pub fn shard_tests(tests: Vec<Test>, shard_time_ms: u32) {
  let mut tests_max_heap: BinaryHeap<(u32, )> = BinaryHeap::new();

  tests_max_heap
}
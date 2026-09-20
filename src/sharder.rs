use std::collections::BinaryHeap;
use crate::test_case::Test;

#[cfg(test)]
mod tests;

pub fn shard_tests(tests: Vec<Test>, shard_time_ms: u32) -> Vec<Vec<Test>> {
  // let shard_time: u32 = 

  let mut tests_max_heap: BinaryHeap<(u32, )> = BinaryHeap::new();

  todo!("not impl");
}

// Original implementation retained only as a differential test oracle.
use std::{
    cmp::max,
    collections::{BTreeMap, BinaryHeap},
};

use crate::test_case::Test;
// use std::collections::BinaryHeap;

pub fn shard_tests(tests: Vec<Test>, target_shard_time_ms: u32) -> Vec<Vec<Test>> {
    let tests_durations: Vec<u32> = tests.iter().map(|test| test.duration_ms).collect();

    let longest_test_duration: u32 = tests_durations.iter().max().copied().unwrap_or(0);
    let shard_time_ms = max(longest_test_duration, target_shard_time_ms);

    let n_shards: usize = tests_durations
        .iter()
        .map(|&duration_ms| u64::from(duration_ms))
        .sum::<u64>()
        .div_ceil(u64::from(shard_time_ms))
        .try_into()
        .expect("shard count does not fit in usize");

    let mut shards: BTreeMap<(u32, usize), Vec<Test>> = (0..n_shards)
        .map(|id| ((shard_time_ms, id), Vec::new()))
        .collect();

    let mut tests_sorted: BinaryHeap<Test> = tests.into();
    while let Some(test) = tests_sorted.pop() {
        let test_duration = test.duration_ms;
        let candidate_key = shards
            .range((test_duration, 0)..)
            .next()
            .map(|(key, _tests)| *key);

        if candidate_key.is_none() {
            shards.insert((shard_time_ms - test_duration, shards.len()), vec![test]);
            continue;
        }

        let Some((remaining_ms, shard_id)) = candidate_key else {
            continue;
        };

        let mut shard_tests = shards.remove(&candidate_key.unwrap()).unwrap();
        shard_tests.push(test);

        shards.insert((remaining_ms - test_duration, shard_id), shard_tests);
    }

    let result: Vec<Vec<Test>> = shards.into_values().collect();
    return result;
}

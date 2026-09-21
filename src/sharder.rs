use std::{cmp::max, collections::BTreeSet};

use crate::test_case::Test;

#[cfg(test)]
mod tests;

pub fn shard_tests(mut tests: Vec<Test>, target_shard_time_ms: u32) -> Vec<Vec<Test>> {
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

    let mut shards: Vec<Vec<Test>> = (0..n_shards).map(|_| Vec::new()).collect();
    let mut available: BTreeSet<(u32, usize)> =
        (0..n_shards).map(|id| (shard_time_ms, id)).collect();

    tests.sort_unstable_by(|a, b| b.cmp(a));
    for test in tests {
        let test_duration = test.duration_ms;
        if let Some((remaining_ms, shard_id)) =
            available.range((test_duration, 0)..).next().copied()
        {
            available.remove(&(remaining_ms, shard_id));
            shards[shard_id].push(test);
            available.insert((remaining_ms - test_duration, shard_id));
        } else {
            let shard_id = shards.len();
            shards.push(vec![test]);
            available.insert((shard_time_ms - test_duration, shard_id));
        }
    }

    available
        .into_iter()
        .map(|(_, shard_id)| std::mem::take(&mut shards[shard_id]))
        .collect()
}

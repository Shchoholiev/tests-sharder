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

    let mut completed = Vec::new();
    let mut lowest_completed: Option<usize> = None;

    tests.sort_unstable_by(|a, b| b.cmp(a));
    for test in tests {
        let test_duration = test.duration_ms;
        // Zero-duration tests must still choose the lowest-ID full shard.
        if test_duration == 0 {
            if let Some(shard_id) = lowest_completed {
                shards[shard_id].push(test);
                continue;
            }
        }
        let (remaining_ms, shard_id) =
            if let Some(key) = available.range((test_duration, 0)..).next().copied() {
                available.remove(&key);
                key
            } else {
                let shard_id = shards.len();
                shards.push(Vec::with_capacity(1));
                (shard_time_ms, shard_id)
            };
        shards[shard_id].push(test);
        let remaining_ms = remaining_ms - test_duration;
        if remaining_ms == 0 {
            lowest_completed = Some(lowest_completed.map_or(shard_id, |old| old.min(shard_id)));
            completed.push(shard_id);
        } else {
            available.insert((remaining_ms, shard_id));
        }
    }

    // Full shards precede all shards with remaining capacity in the result.
    completed.sort_unstable();
    completed
        .into_iter()
        .chain(available.into_iter().map(|(_, shard_id)| shard_id))
        .map(|shard_id| std::mem::take(&mut shards[shard_id]))
        .collect()
}

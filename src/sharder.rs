use std::{cmp::max, collections::BTreeSet};

use crate::test_case::Test;

#[cfg(test)]
mod tests;

pub fn shard_tests(mut tests: Vec<Test>, target_shard_time_ms: u32) -> Vec<Vec<Test>> {
    let tests_durations: Vec<u32> = tests.iter().map(|test| test.duration_ms.get()).collect();

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
    let mut available: BTreeSet<u64> = (0..n_shards)
        .map(|id| pack_shard_key(shard_time_ms, id))
        .collect();

    let mut completed = Vec::new();

    // Longest tests first, with higher IDs breaking duration ties.
    tests.sort_unstable_by(|a, b| b.cmp(a));
    for test in tests {
        let duration_ms = test.duration_ms.get();
        let (shard_id, free_before_ms) =
            take_best_fit_or_create_shard(&mut available, &mut shards, duration_ms, shard_time_ms);

        shards[shard_id].push(test);
        let remaining_ms = free_before_ms - duration_ms;
        if remaining_ms == 0 {
            completed.push(shard_id);
        } else {
            available.insert(pack_shard_key(remaining_ms, shard_id));
        }
    }

    // Full shards precede all shards with remaining capacity in the result.
    completed.sort_unstable();
    completed
        .into_iter()
        .chain(available.into_iter().map(|key| unpack_shard_key(key).1))
        .map(|shard_id| std::mem::take(&mut shards[shard_id]))
        .collect()
}

fn take_best_fit_or_create_shard(
    available: &mut BTreeSet<u64>,
    shards: &mut Vec<Vec<Test>>,
    duration_ms: u32,
    shard_time_ms: u32,
) -> (usize, u32) {
    if let Some(key) = available
        .extract_if(pack_shard_key(duration_ms, 0).., |_| true)
        .next()
    {
        let (free_before_ms, shard_id) = unpack_shard_key(key);
        (shard_id, free_before_ms)
    } else {
        let shard_id = shards.len();
        shards.push(Vec::with_capacity(1));
        (shard_id, shard_time_ms)
    }
}

fn pack_shard_key(remaining_ms: u32, shard_id: usize) -> u64 {
    // Pack remaining time and shard ID into 8 bytes; a (u32, u32) key benchmarked 7–11% slower.
    (u64::from(remaining_ms) << 32) | shard_id as u64
}

fn unpack_shard_key(key: u64) -> (u32, usize) {
    ((key >> 32) as u32, key as u32 as usize)
}

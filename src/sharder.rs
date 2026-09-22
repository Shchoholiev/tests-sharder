use std::{cmp::max, collections::BTreeSet};

use crate::test_case::Test;

#[cfg(test)]
mod tests;

pub fn shard_tests(mut tests: Vec<Test>, target_shard_time_ms: u32) -> Vec<Vec<Test>> {
    assert!(
        tests.len() <= u32::MAX as usize,
        "packed shard keys support at most u32::MAX tests"
    );
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

    // With at most one shard, sorted input is already the complete result.
    if n_shards <= 1 {
        if tests.is_empty() {
            return Vec::new();
        }
        sort_tests(&mut tests);
        return vec![tests];
    }

    let mut shards: Vec<Vec<Test>> = (0..n_shards).map(|_| Vec::new()).collect();
    let mut available: BTreeSet<u64> = (0..n_shards)
        .map(|id| pack_shard_key(shard_time_ms, id))
        .collect();

    let mut completed = Vec::new();
    let mut lowest_completed: Option<usize> = None;

    sort_tests(&mut tests);
    for test in tests {
        let test_duration = test.duration_ms;
        // Zero-duration tests must still choose the lowest-ID full shard.
        if test_duration == 0 {
            if let Some(shard_id) = lowest_completed {
                shards[shard_id].push(test);
                continue;
            }
        }
        let (remaining_ms, shard_id) = if let Some(key) = available
            .extract_if(pack_shard_key(test_duration, 0).., |_| true)
            .next()
        {
            unpack_shard_key(key)
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

fn pack_shard_key(remaining_ms: u32, shard_id: usize) -> u64 {
    // Capacity in the high bits orders shards by remaining time, then ID.
    (u64::from(remaining_ms) << 32) | shard_id as u64
}

fn unpack_shard_key(key: u64) -> (u32, usize) {
    ((key >> 32) as u32, key as u32 as usize)
}

fn sort_tests(tests: &mut [Test]) {
    tests.sort_unstable_by(|a, b| b.cmp(a));
}

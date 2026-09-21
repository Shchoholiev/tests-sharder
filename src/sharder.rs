use std::{cmp::max, collections::BTreeSet};

use crate::test_case::Test;

#[cfg(test)]
mod tests;

pub fn shard_tests(tests: Vec<Test>, target_shard_time_ms: u32) -> Vec<Vec<Test>> {
    // Shard IDs are below the input length; larger inputs retain full-width IDs.
    if tests.len() <= u32::MAX as usize {
        shard_tests_with_key::<u64>(tests, target_shard_time_ms)
    } else {
        shard_tests_with_key::<(u32, usize)>(tests, target_shard_time_ms)
    }
}

fn shard_tests_with_key<K: ShardKey>(
    mut tests: Vec<Test>,
    target_shard_time_ms: u32,
) -> Vec<Vec<Test>> {
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
    let mut available: BTreeSet<K> = (0..n_shards).map(|id| K::new(shard_time_ms, id)).collect();

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
            .extract_if(K::new(test_duration, 0).., |_| true)
            .next()
        {
            key.parts()
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
            available.insert(K::new(remaining_ms, shard_id));
        }
    }

    // Full shards precede all shards with remaining capacity in the result.
    completed.sort_unstable();
    completed
        .into_iter()
        .chain(available.into_iter().map(|key| key.parts().1))
        .map(|shard_id| std::mem::take(&mut shards[shard_id]))
        .collect()
}

trait ShardKey: Copy + Ord {
    fn new(remaining_ms: u32, shard_id: usize) -> Self;
    fn parts(self) -> (u32, usize);
}

impl ShardKey for u64 {
    fn new(remaining_ms: u32, shard_id: usize) -> Self {
        debug_assert!(shard_id <= u32::MAX as usize);
        // Capacity in the high bits preserves the original tuple ordering.
        (u64::from(remaining_ms) << 32) | shard_id as u64
    }

    fn parts(self) -> (u32, usize) {
        ((self >> 32) as u32, self as u32 as usize)
    }
}

impl ShardKey for (u32, usize) {
    fn new(remaining_ms: u32, shard_id: usize) -> Self {
        (remaining_ms, shard_id)
    }

    fn parts(self) -> (u32, usize) {
        self
    }
}

fn sort_tests(tests: &mut [Test]) {
    // The extra grouping pass costs more than it saves on small inputs.
    if tests.len() <= 1_024 {
        tests.sort_unstable_by(|a, b| b.cmp(a));
        return;
    }
    tests.sort_unstable_by_key(|test| std::cmp::Reverse(test.duration_ms));
    let mut remaining = tests;
    while let Some(first) = remaining.first() {
        let duration = first.duration_ms;
        let end = remaining
            .iter()
            .position(|test| test.duration_ms != duration)
            .unwrap_or(remaining.len());
        let (equal_duration, rest) = remaining.split_at_mut(end);
        equal_duration.sort_unstable_by(|a, b| b.id.cmp(&a.id));
        remaining = rest;
    }
}

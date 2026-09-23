use super::*;

// If sharding uses greedy algorithm it would split given tests into:
// 1) 3, 2
// 2) 3, 2
// 3) 2
// While 2 shards can already fit 12 ms of tests
#[test]
fn greedy_edge_case_returns_two_shards() {
    let tests = vec![
        Test::new("1", 3),
        Test::new("2", 2),
        Test::new("3", 3),
        Test::new("4", 2),
        Test::new("5", 2),
    ];
    let shard_time_ms = 6;

    let shards = shard_tests(tests.clone(), shard_time_ms);

    assert!(shards.len() == 2);
    assert_eq!(max_shard_duration(&shards), 6);
    assert_tests_integrity(tests, &shards)
}

// When the longest test in the list is greater than the requested time
// it makes no sense to keep one shard run longer than others,
// so we should increase time of each shards to reduce number of shards
// Negative case:
// 1) 6
// 2) 3
// 3) 3
#[test]
fn test_longer_than_requested_time_returns_longer_shards() {
    let tests = vec![Test::new("1", 6), Test::new("2", 3), Test::new("3", 3)];
    let shard_time_ms = 5;

    let shards = shard_tests(tests.clone(), shard_time_ms);

    assert!(shards.len() == 2);
    assert_eq!(max_shard_duration(&shards), 6);
    assert!(
        shards[0]
            .iter()
            .map(|test| test.duration_ms.get())
            .sum::<u32>()
            == 6
    );
    assert!(
        shards[1]
            .iter()
            .map(|test| test.duration_ms.get())
            .sum::<u32>()
            == 6
    );
    assert_tests_integrity(tests, &shards)
}

// Sum of tests duration may fit in the allocted shards,
// but we might not be able to split job into this number of shards,
// so we need to add extra shard
// tests = [4, 3, 3], time = 5
// 10 / 5 = 2 shards
// 1) 4
// 2) 3
// 3) 3 - extra shard
#[test]
fn tests_duration_requires_extra_shard() {
    let tests = vec![Test::new("1", 3), Test::new("2", 4), Test::new("3", 3)];
    let shard_time_ms = 5;

    let shards = shard_tests(tests.clone(), shard_time_ms);

    assert!(shards.len() == 3);
    assert_eq!(max_shard_duration(&shards), 4);
    assert_tests_integrity(tests, &shards)
}

#[test]
fn tests_that_fit_return_one_shard() {
    let tests = vec![Test::new("1", 2), Test::new("2", 1), Test::new("3", 3)];
    let shard_time_ms = 6;

    let shards = shard_tests(tests.clone(), shard_time_ms);

    assert!(shards.len() == 1);
    assert_eq!(max_shard_duration(&shards), 6);
    assert_tests_integrity(tests, &shards)
}

#[test]
fn empty_input_returns_no_shards() {
    assert!(shard_tests(Vec::new(), 5).is_empty());
}

#[test]
fn total_duration_can_exceed_u32_max() {
    let tests = vec![Test::new("1", u32::MAX), Test::new("2", u32::MAX)];

    let expected = tests.clone();

    let shards = shard_tests(tests, u32::MAX);

    assert_eq!(shards.len(), 2);
    assert_eq!(max_shard_duration(&shards), u64::from(u32::MAX));
    assert_tests_integrity(expected, &shards);
}

fn assert_tests_integrity(expected: Vec<Test>, shards: &[Vec<Test>]) {
    let mut expected_clone = expected.clone();
    let mut actual: Vec<Test> = shards.iter().flatten().cloned().collect();

    actual.sort();
    expected_clone.sort();

    assert!(actual == *expected_clone)
}

fn max_shard_duration(shards: &[Vec<Test>]) -> u64 {
    shards
        .iter()
        .map(|shard| {
            shard
                .iter()
                .map(|test| u64::from(test.duration_ms.get()))
                .sum()
        })
        .max()
        .unwrap_or(0)
}

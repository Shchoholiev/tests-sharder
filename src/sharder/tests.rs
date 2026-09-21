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
    let tests = vec![
        Test::new("1", 6),
        Test::new("2", 3),
        Test::new("3", 3),
    ];
    let shard_time_ms = 5;

    let shards = shard_tests(tests.clone(), shard_time_ms);

    assert!(shards.len() == 2);
    assert!(shards[0].iter().map(|test| test.duration_ms).sum::<u32>() == 6);
    assert!(shards[1].iter().map(|test| test.duration_ms).sum::<u32>() == 6);
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
    let tests = vec![
        Test::new("1", 3),
        Test::new("2", 4),
        Test::new("3", 3),
    ];
    let shard_time_ms = 5;

    let shards = shard_tests(tests.clone(), shard_time_ms);

    assert!(shards.len() == 3);
    assert_tests_integrity(tests, &shards)
}

#[test]
fn tests_that_fit_return_one_shard() {
    let tests = vec![
        Test::new("1", 2),
        Test::new("2", 1),
        Test::new("3", 3),
    ];
    let shard_time_ms = 6;

    let shards = shard_tests(tests.clone(), shard_time_ms);

    assert!(shards.len() == 1);
    assert_tests_integrity(tests, &shards)
}

#[test]
fn empty_input_returns_no_shards() {
    assert!(shard_tests(Vec::new(), 5).is_empty());
}

#[test]
fn total_duration_can_exceed_u32_max() {
    let tests = vec![
        Test::new("1", u32::MAX),
        Test::new("2", u32::MAX),
    ];

    let expected = tests.clone();

    let shards = shard_tests(tests, u32::MAX);

    assert_eq!(shards.len(), 2);
    assert_tests_integrity(expected, &shards);
}

fn assert_tests_integrity(expected: Vec<Test>, shards: &Vec<Vec<Test>>) {
    let mut expected_clone = expected.clone();
    let mut actual: Vec<Test> = shards.iter().flatten().cloned().collect();

    actual.sort();
    expected_clone.sort();

    assert!(actual == *expected_clone)
}

#[path = "reference.rs"]
mod reference;

fn check_equivalence(tests: Vec<Test>, target: u32) {
    let capacity = target.max(tests.iter().map(|t| t.duration_ms).max().unwrap_or(0));
    let expected = reference::shard_tests(tests.clone(), target);
    let wide = shard_tests_with_key::<(u32, usize)>(tests.clone(), target);
    assert!(
        wide == expected,
        "wide-key output differs for target {target}"
    );
    let actual = shard_tests(tests.clone(), target);
    assert!(
        actual == expected,
        "ordered output differs for target {target}"
    );
    assert!(actual.iter().all(|shard| {
        shard.iter().map(|t| u64::from(t.duration_ms)).sum::<u64>() <= u64::from(capacity)
    }));
    assert_tests_integrity(tests, &actual);
}

#[test]
fn exhaustive_small_inputs_preserve_exact_output() {
    for len in 0..=6 {
        for mut encoded in 0..4usize.pow(len) {
            let tests: Vec<_> = (0..len)
                .map(|index| {
                    let duration = (encoded % 4) as u32;
                    encoded /= 4;
                    Test::new(
                        [
                            "2", "10", "2",
                        ][index as usize % 3],
                        duration,
                    )
                })
                .collect();
            for target in 0..=6 {
                if target == 0 && tests.iter().all(|t| t.duration_ms == 0) {
                    continue;
                }
                check_equivalence(tests.clone(), target);
            }
        }
    }
}

#[test]
fn randomized_inputs_preserve_exact_output() {
    use rand::{Rng, SeedableRng, rngs::StdRng};
    let mut rng = StdRng::seed_from_u64(77);
    for _ in 0..500 {
        let tests = (0..rng.random_range(0..200))
            .map(|_| {
                Test::new(
                    rng.random_range(0..30).to_string(),
                    rng.random_range(0..1000),
                )
            })
            .collect();
        check_equivalence(tests, rng.random_range(1..3000));
    }
    check_equivalence(
        vec![
            Test::new("a", u32::MAX),
            Test::new("b", u32::MAX),
            Test::new("z", 0),
        ],
        0,
    );
}

#[test]
#[should_panic]
fn empty_input_with_zero_target_panics() {
    shard_tests(vec![], 0);
}

#[test]
#[should_panic]
fn zero_durations_with_zero_target_panic() {
    shard_tests(vec![Test::new("zero", 0)], 0);
}

#[test]
fn packed_keys_preserve_tuple_order_and_boundaries() {
    let mut pairs = Vec::new();
    for remaining in [
        0,
        1,
        u32::MAX,
    ] {
        for id in [
            0,
            1,
            u32::MAX as usize,
        ] {
            let key = <u64 as ShardKey>::new(remaining, id);
            assert_eq!(key.parts(), (remaining, id));
            pairs.push((remaining, id));
        }
    }
    use rand::{Rng, SeedableRng, rngs::StdRng};
    let mut rng = StdRng::seed_from_u64(99);
    for _ in 0..1000 {
        pairs.push((rng.random::<u32>(), rng.random::<u32>() as usize));
    }
    let mut packed: Vec<_> = pairs
        .iter()
        .map(|&(remaining, id)| <u64 as ShardKey>::new(remaining, id))
        .collect();
    pairs.sort_unstable();
    packed.sort_unstable();
    assert_eq!(
        packed.into_iter().map(ShardKey::parts).collect::<Vec<_>>(),
        pairs
    );
}

#[test]
#[cfg(target_pointer_width = "64")]
fn wide_keys_preserve_large_shard_ids() {
    let id = u32::MAX as usize + 1;
    let key = <(u32, usize) as ShardKey>::new(u32::MAX, id);
    assert_eq!(key.parts(), (u32::MAX, id));
    assert!(key > (u32::MAX, id - 1));
}

#[test]
fn sorting_paths_preserve_output_around_small_input_cutoff() {
    for count in [
        1_023, 1_024, 1_025, 10_000,
    ] {
        for repeated_durations in [
            true, false,
        ] {
            let tests: Vec<_> = (0..count)
                .map(|index| {
                    let duration = if repeated_durations {
                        index % 13
                    } else {
                        index
                    };
                    Test::new((index % 57).to_string(), duration as u32)
                })
                .collect();
            check_equivalence(tests.clone(), 777);
            check_equivalence(tests, u32::MAX);
        }
    }
}

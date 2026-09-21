use std::{hint::black_box, time::Duration};

use criterion::{
    BatchSize, BenchmarkGroup, Criterion, SamplingMode, Throughput, criterion_group,
    criterion_main, measurement::WallTime,
};
use rand::{Rng, SeedableRng, rngs::StdRng};
use tests_sharder::{sharder::shard_tests, test_case::Test};

fn benchmark_sharding_regular(c: &mut Criterion) {
    let mut group = c.benchmark_group("sharding_regular");
    group.measurement_time(Duration::from_secs(10));

    benchmark_counts(
        &mut group,
        &[
            1_000, 10_000, 100_000,
        ],
    );

    group.finish();
}

fn benchmark_sharding_large(c: &mut Criterion) {
    let mut group = c.benchmark_group("sharding_large");
    group.sample_size(10);
    group.warm_up_time(Duration::from_secs(1));
    group.measurement_time(Duration::from_secs(100));
    group.sampling_mode(SamplingMode::Flat);

    benchmark_counts(
        &mut group,
        &[
            1_000_000, 10_000_000,
        ],
    );

    group.finish();
}

fn benchmark_counts(group: &mut BenchmarkGroup<'_, WallTime>, counts: &[usize]) {
    for &count in counts {
        group.throughput(Throughput::Elements(count as u64));

        group.bench_with_input(format!("{count}_tests"), &count, |bencher, &count| {
            bencher.iter_batched(
                || generate_tests(count),
                |tests| {
                    black_box(shard_tests(
                        black_box(tests),
                        black_box(300_000), // 5 min
                    ))
                },
                BatchSize::LargeInput,
            )
        });
    }
}

fn generate_tests(count: usize) -> Vec<Test> {
    let mut rng = StdRng::seed_from_u64(42);

    (0..count)
        .map(|index| {
            // 1 second to 10 min
            let duration_ms = rng.random_range(1_000..600_000);
            Test::new(index.to_string(), duration_ms)
        })
        .collect()
}

fn benchmark_sharding_shapes(c: &mut Criterion) {
    let mut group = c.benchmark_group("sharding_shapes");
    group.measurement_time(Duration::from_secs(10));
    group.throughput(Throughput::Elements(100_000));
    for shape in [
        "short",
        "half_capacity",
        "ties",
        "skewed",
    ] {
        group.bench_function(shape, |bencher| {
            bencher.iter_batched(
                || {
                    let mut rng = StdRng::seed_from_u64(42);
                    (0..100_000)
                        .map(|index| {
                            let duration = match shape {
                                "short" => rng.random_range(1..1000),
                                "half_capacity" => rng.random_range(149_999..150_002),
                                "ties" => 100_000,
                                "skewed" => {
                                    if index % 100 == 0 {
                                        299_999
                                    } else {
                                        rng.random_range(1..1000)
                                    }
                                }
                                _ => unreachable!(),
                            };
                            Test::new(index.to_string(), duration)
                        })
                        .collect()
                },
                |tests| black_box(shard_tests(black_box(tests), black_box(300_000))),
                BatchSize::LargeInput,
            );
        });
    }
    group.finish();
}

fn benchmark_sharding_single(c: &mut Criterion) {
    let mut group = c.benchmark_group("sharding_single");
    group.measurement_time(Duration::from_secs(10));
    for count in [
        1_000usize, 100_000,
    ] {
        group.throughput(Throughput::Elements(count as u64));
        group.bench_function(format!("{count}_tests"), |bencher| {
            bencher.iter_batched(
                || {
                    let mut rng = StdRng::seed_from_u64(42);
                    (0..count)
                        .map(|index| Test::new(index.to_string(), rng.random_range(1..1000)))
                        .collect()
                },
                |tests| black_box(shard_tests(black_box(tests), black_box(100_000_000))),
                BatchSize::LargeInput,
            );
        });
    }
    group.finish();
}

criterion_group!(
    sharding,
    benchmark_sharding_regular,
    benchmark_sharding_large,
    benchmark_sharding_shapes,
    benchmark_sharding_single
);
criterion_main!(sharding);

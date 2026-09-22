use std::{hint::black_box, time::Duration};

use criterion::{
    criterion_group, criterion_main, measurement::WallTime, BatchSize, BenchmarkGroup, Criterion,
    SamplingMode, Throughput,
};
use rand::{rngs::StdRng, Rng, SeedableRng};
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

criterion_group!(
    sharding,
    benchmark_sharding_regular,
    benchmark_sharding_large
);
criterion_main!(sharding);

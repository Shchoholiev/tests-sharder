use std::{hint::black_box, time::Duration};

use criterion::{Criterion, Throughput, criterion_group, criterion_main, BatchSize};
use rand::{Rng, SeedableRng, rngs::StdRng};
use tests_sharder::{sharder::shard_tests, test_case::Test};

fn benchmark_sharding(c: &mut Criterion) {
    let mut group = c.benchmark_group("tests_sharding");
    group.measurement_time(Duration::from_secs(10));

    for count in [1_000usize, 10_000, 100_000] {
        group.throughput(
            Throughput::Elements(count as u64),
        );

        group.bench_with_input(
            format!("{count}_tests"), 
            &count, 
            |bencher, &count| {
                bencher.iter_batched(
                    || generate_tests(count),
                    |tests| 
                        black_box(
                            shard_tests(
                                black_box(tests), 
                                black_box(300_000) // 5 min
                            ) 
                        ), 
                    BatchSize::LargeInput
                )
            }
        );
    }
}

fn generate_tests(count: usize) -> Vec<Test> {
    let mut rng = StdRng::seed_from_u64(42);

    (0..count)
        .map(|index| {
            // 1 second to 30 min
            let duration_ms = rng.random_range(1_000..1800_000);
            Test::new(index.to_string(), duration_ms)
        })
        .collect()
}

criterion_group!(sharding, benchmark_sharding);
criterion_main!(sharding);
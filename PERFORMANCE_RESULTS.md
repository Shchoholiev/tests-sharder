# Performance results

Branch: `codex/sharding-performance`, based on main `63defa5`.
Rust/Cargo 1.98.1, aarch64-apple-darwin; locked dependencies and release benchmarks.

## Decisions

- **Keep in-place sorting.** All nine workloads improved. Compared with the original: 100K **22.7% faster** (95% CI 22.4–23.1%); 1M **46.4% faster** (46.0–46.9%); 10M **68.3% faster** (67.6–69.1%). Exact output is preserved.
- **Discard fused max/sum.** Removed the duration buffer, but 10M slowed **10.4%** (95% CI 7.5–12.8%). The 1M result was inconclusive. Conservatively rejected after one comparison; the cause was not established.
- **Discard lazy empty-shard creation.** Versus sorting alone, improved 1M by 7.5% and 10M by 5.7%, below the 10% gate. It also slowed tied durations by **7.2%** (95% CI 5.9–8.7%) and skewed inputs by **7.0%** (6.2–7.4%). No repeat was needed to resolve the missed improvement gate.

## Retained change: measured times

| Workload | Original | Sorting |
|---|---:|---:|
| 1K | 0.091 ms | 0.087 ms |
| 10K | 1.753 ms | 1.589 ms |
| 100K | 24.097 ms | 18.624 ms |
| 1M | 406.205 ms | 217.542 ms |
| 10M | 7946.977 ms | 2522.711 ms |
| Short, 100K | 15.427 ms | 8.879 ms |
| Half capacity, 100K | 26.813 ms | 18.813 ms |
| Ties, 100K | 25.228 ms | 18.255 ms |
| Skewed, 100K | 16.905 ms | 10.189 ms |

Times use Criterion point estimates; percentage comparisons above use Criterion's comparison analysis. The original benchmark settings/generator are unchanged. Added 100K workloads cover short durations, values around half capacity, ties, and a skewed mixture.

## Validation and reproduction

All 10 tests pass in debug and release. Differential tests cover **38,220** valid exhaustive small cases and **501** randomized/boundary cases, comparing exact nested output with the original. Independent checks enforce test preservation and shard capacity. Zero-capacity panic behavior stays unchanged. Formatting and binary build checks pass.

Local Criterion baselines: `stage-0` original, `stage-1` fused scan (rejected), `stage-2` sort (retained), `stage-3` sort plus lazy creation (rejected). Stored under `target/criterion`, not committed.

```sh
cargo test --locked
cargo test --locked --release
cargo bench --locked --bench sharding -- --save-baseline NAME
cargo bench --locked --bench sharding -- --load-baseline stage-2 --baseline stage-0
```

`--load-baseline` compares saved measurements without resampling. Benchmark sampling runs were sequential, without concurrent builds/tests. These are local measurements; confidence intervals do not rule out machine drift.

A separate three-run 1M timing probe after sorting measured metadata 1.3–1.5 ms, initial tree 5.9–6.4 ms, sorting 46–47 ms, placement 165–171 ms, and output 7.4–9.3 ms. This motivated the lazy-creation experiment; it is not part of Criterion timing. No peak-memory improvement is claimed.

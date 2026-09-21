# Performance spec — pending review

Optimize one change at a time; begin implementation after review.

## Baseline

- All six unit tests pass. Criterion baseline `pre-opt`: **90 µs / 1.75 ms / 23.88 ms** for **1K / 10K / 100K** tests. Large cases remain unmeasured.
- Benchmark: seed 42, durations `[1,000, 600,000)` ms, sizes 1K–10M. Input generation and returned-output destruction are outside timing.
- The five-minute target expands to nearly ten minutes, producing roughly `n/2` shards.
- Current best-fit decreasing algorithm costs `O(n log n + n log S)` for `n` tests and `S` shards: heap pops plus tree lookup/remove/insert per test.

## Correctness

Preserve exact output: every test occurrence, shard membership, ordering, and tie-breaking. Keep capacity `max(target, longest test)`, `u64` totals, and checked shard-count conversion. Preserve the existing zero-capacity panic; fix it separately.

Before optimizing, add differential tests against the original implementation for ties, duplicates, zeros, exact fits, extra shards, and large totals. Independently check test preservation and shard capacity.

## Stages

| Change | Why | Acceptance |
|---|---|---|
| 1. Compute max and sum in one pass | Removes duration buffer and extra scans; saves ~40 MB at 10M tests | Allocation removed; no repeatable slowdown >3% |
| 2. Sort in place instead of draining a heap | Potentially better locality with identical descending `(duration, id)` order | ≥5% faster at 100K and 1M |
| 3. Materialize empty shards when selected | Avoids building roughly `n/2` empty tree entries upfront | ≥10% faster at 1M and 10M |

Profile before stage 3 to confirm tree work warrants it. Preserve logical shard IDs and final ordering when creating entries lazily.

## Validation loop

1. Establish large-input baselines and separately named short-test, repeated-duration, and skewed workloads before changing the algorithm.
2. Implement one stage; run `cargo test --locked` in debug and release.
3. Compare Criterion against the previous accepted baseline and original baseline on the same machine and harness.
4. Require exact correctness, no repeatable slowdown >3% on other cases, and 95% comparison intervals below zero for claimed speedups. Repeat noisy results.
5. Review evidence before the next stage. Percentage targets are acceptance gates, not promised gains.

Save: `cargo bench --locked --bench sharding -- --save-baseline NAME`

Compare: `cargo bench --locked --bench sharding -- --baseline NAME`

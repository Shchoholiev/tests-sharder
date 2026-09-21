# Performance experiments

Branch: `codex/sharding-performance`, based on main `63defa5`.
Rust/Cargo 1.98.1, aarch64-apple-darwin; locked dependencies, release benchmarks.

## Method

Each candidate passes debug/release tests before measurement. Tests compare exact nested output with the original implementation over 38,220 valid small cases and 501 random/boundary cases, plus existing tests and zero-capacity panic checks.

Criterion settings and the original generator are unchanged. Added 100K-test workloads cover short durations, values around half capacity, equal durations, and a skewed mixture. Each stage has a named baseline under `target/criterion` (local, untracked).

Commands:

```sh
cargo test --locked
cargo test --locked --release
cargo bench --locked --bench sharding -- --save-baseline stage-N
cargo bench --locked --bench sharding -- --load-baseline stage-N --baseline stage-PREV
```

`--load-baseline` compares stored measurements without sampling again. Runs are sequential. Results reflect this machine and these inputs; confidence intervals do not eliminate machine drift.

## Results

- **Stage 1 (fused max/sum): rejected.** Removes `4n` bytes of temporary storage, but the 10M case rose from 7.947 s to 8.776 s: +10.44% (95% comparison interval +7.53% to +12.84%). The 1M change was inconclusive; smaller cases ranged from modest gains to a 1.65% short-test regression. This is one measured comparison, not proof of the cause; the candidate was conservatively discarded without a repeat.
- Stage 2 tests sorting against stage 0, since stage 1 was discarded.

- **Stage 2 (in-place sort): accepted.** All nine workloads improved. At 100K: -22.71% (95% CI -23.07% to -22.38%); 1M: -46.45% (-46.94% to -46.02%); 10M: -68.26% (-69.10% to -67.58%).
- Standalone phase timing after sorting (three runs, seed-42 1M workload): metadata 1.3–1.5 ms, initial tree 5.9–6.4 ms, sort 46–47 ms, placement 165–171 ms, output 7.4–9.3 ms. Placement remains the main target. These diagnostic timings are separate from Criterion measurements.

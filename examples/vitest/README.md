# Vitest sharding example

Rust plans once. Vitest's custom sequencer reads the saved plan and runs one
assigned shard per process.

From the repository root, with Rust, Node.js, and npm installed:

```sh
cd examples/vitest
npm install --no-package-lock
cargo build --locked --manifest-path ../../Cargo.toml
./shard.sh plan test-durations.jsonl 1000 shard-plan.jsonl
./shard.sh run shard-plan.jsonl 1
./shard.sh run shard-plan.jsonl 2
```

The four tests really wait 600, 500, 500, and 400 ms, matching
[test-durations.jsonl](test-durations.jsonl). Rust's 1,000 ms target produces two
balanced shards. The [script](shard.sh) can run in separate CI jobs: `plan`
saves Rust's output, and `run` selects one shard from that file. Set
`TESTS_SHARDER_BIN` if the binary is elsewhere; pass additional Vitest options
after the shard index.
Build the CI matrix from the number of lines in the plan and share that file
with every shard job. For affected-test runs, give `plan` JSONL for the selected
files. Vitest's `--shard` calls the
[sequencer](vitest.config.ts), which reads `SHARD_PLAN` and returns the assigned
files.

# tests-sharder

## Download a release

Download the archive for your operating system and CPU from the
[GitHub Releases page](https://github.com/Shchoholiev/tests-sharder/releases).
Each release includes these six targets:

| OS | CPU | Archive suffix |
| --- | --- | --- |
| Linux | x86-64 | `x86_64-unknown-linux-gnu.tar.gz` |
| Linux | ARM64 | `aarch64-unknown-linux-gnu.tar.gz` |
| macOS | Intel | `x86_64-apple-darwin.tar.gz` |
| macOS | Apple Silicon | `aarch64-apple-darwin.tar.gz` |
| Windows | x86-64 | `x86_64-pc-windows-msvc.zip` |
| Windows | ARM64 | `aarch64-pc-windows-msvc.zip` |

For example, a Linux x86-64 project can pin a release and download its binary:

```sh
VERSION=v0.1.0
curl -fsSLO "https://github.com/Shchoholiev/tests-sharder/releases/download/$VERSION/tests-sharder-x86_64-unknown-linux-gnu.tar.gz"
tar -xzf tests-sharder-x86_64-unknown-linux-gnu.tar.gz
./tests-sharder --help
```

Use the matching archive suffix on other platforms. Archives contain a single
`tests-sharder` executable (`tests-sharder.exe` on Windows). `SHA256SUMS` in each
release lists checksums for all six archives. Pinning `VERSION` makes project
builds repeatable. To download the newest release instead, use
`https://github.com/Shchoholiev/tests-sharder/releases/latest/download/tests-sharder-x86_64-unknown-linux-gnu.tar.gz`
with the suffix for your platform.

## CLI

Provide one test per JSONL line, with an `id` and a positive `duration_ms`:

```json
{"id":"login","duration_ms":1200}
```

Read from a file or pipe input through stdin. The CLI writes one JSONL line per
shard to stdout:

```sh
tests-sharder --target-ms 60000 tests.jsonl > shards.jsonl
cat tests.jsonl | tests-sharder --target-ms 60000
```

## Development setup

Install Rust's standard formatter and linter, then enable the repository's Git hooks:

```sh
rustup component add rustfmt clippy
git config --local core.hooksPath .githooks
```

Run `cargo fmt` to format the Rust source, or `cargo fmt --all -- --check`
to check formatting without changing files.

The pre-commit hook runs `cargo fmt --all -- --check`, Clippy across all targets
and features with warnings treated as errors, and `cargo build --locked --bins`.
It blocks the commit if any check fails. It checks the current working tree,
including unstaged changes. Enable the hooks once for each clone of this
repository.

## Publishing a release

After merging changes to the default branch, set the version in `Cargo.toml`
and update `Cargo.lock` with `cargo check`. Then tag that commit and push the tag:

```sh
git tag v0.1.0
git push origin v0.1.0
```

Use the version you set in `Cargo.toml` in place of `v0.1.0`. The
[release workflow](.github/workflows/release.yml) checks that the tag matches
the package version, tests and builds all six native targets, and publishes
the archives and checksums as a GitHub Release. A failed build prevents the
release from being published.

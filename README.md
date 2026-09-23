# tests-sharder

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

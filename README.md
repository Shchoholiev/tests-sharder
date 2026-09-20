# tests-sharder

## Development setup

Install Rust's standard formatter and enable the repository's Git hooks:

```sh
rustup component add rustfmt
git config --local core.hooksPath .githooks
```

Run `cargo fmt` to format the Rust source, or `cargo fmt --all -- --check`
to check formatting without changing files.

The pre-commit hook runs `cargo fmt --all -- --check` and
`cargo build --locked --bins`. It blocks the commit if formatting is needed or
the binary fails to compile. It checks the current working tree, including
unstaged changes. Enable the hooks once for each clone of this repository.

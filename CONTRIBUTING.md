# Contributing

apus-rs is a board-support crate for the APUS board. Here's how to work on it.

## Development setup

See [docs/setup.md](docs/setup.md) for the toolchain, probe-rs, and flashing setup.

## Before you open a PR

CI runs these three checks — run them locally first so your PR goes green:

```bash
cargo fmt --all -- --check
cargo clippy --lib --examples -- -D warnings
cargo build --release --examples
```

CI runs them with `--locked` against the committed `Cargo.lock`.

## Pull requests

Branch off `main`, open a pull request (CI runs on every PR), and keep each PR
focused on one change.

## License

By contributing, you agree that your contributions are dual-licensed under MIT
and Apache-2.0, as described in the [README](README.md#license).

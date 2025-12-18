# Contributing

Thanks for your interest in contributing to **Tuitar**! 🎸

Before you start:

- For larger changes, open an issue first (or comment on an existing one) to discuss scope and approach.
- Please read and follow our [Code of Conduct](./CODE_OF_CONDUCT.md).

## Development Setup

1. Ensure you have [Rust](https://www.rust-lang.org/tools/install) installed.

2. Follow the [firmware documentation](firmware/README.md) for setting up the ESP32 toolchain.

3. Clone the repository:

```sh
git clone https://github.com/orhun/tuitar
```

4. You can use the commands below to build, run, and test the project:

```sh
# Build the firmware
cd firmware/
cargo build
```

```sh
# Run the terminal app
cargo run -p tuitar
```

```sh
# Formatting
cargo fmt --all -- --check
```

```sh
# Lints
cargo clippy --all-targets --all-features -- -D warnings
```

## Pull Requests

Please ensure:

- [ ] The change is focused and clearly described.
- [ ] `cargo fmt`, `cargo test`, and `cargo clippy` pass locally (where applicable).
- [ ] New behavior is covered by tests when practical.
- [ ] Docs are updated if the change affects usage, configuration, or hardware/firmware flows.

## Reporting Issues

When filing a bug report, include:

- What you expected vs what happened
- Reproduction steps (as minimal as possible)
- OS / Rust version / device details (ESP32 + board type) when relevant
- Logs or screenshots/videos when helpful

## License

This project is dual-licensed under the [Apache-2.0](./LICENSE-APACHE) or [MIT](./LICENSE-MIT) licenses.
By contributing, you agree that your contributions are licensed under the same terms.

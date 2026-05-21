# Rust Toolchain Modernization

Sprint 76 pins the project to the latest locally selected stable Rust toolchain in `rust-toolchain.toml`.

- target channel stays **stable**
- `rustfmt` and `clippy` remain required components
- `profile = "minimal"` stays in place
- nightly is not required and is rejected by config validation

Rollback is simple: restore the previous `rust-toolchain.toml` channel and rerun the existing workspace checks.

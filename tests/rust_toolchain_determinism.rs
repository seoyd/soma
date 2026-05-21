#[path = "support/sprint69_support.rs"]
mod support;

#[test]
fn sprint76_outputs_are_deterministic() {
    let first = support::run_sprint76_bundle(
        "soma_rust_toolchain_modernize.toml",
        "rust-toolchain-determinism-a",
    );
    let second = support::run_sprint76_bundle(
        "soma_rust_toolchain_modernize.toml",
        "rust-toolchain-determinism-b",
    );
    assert_eq!(first, second);
}

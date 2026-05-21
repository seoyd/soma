#[path = "support/sprint69_support.rs"]
mod support;

use std::fs;

use soma_zero::{RustToolchainChannel, RustToolchainModernizationConfig};

#[test]
fn sprint76_config_defaults_and_example_bundle_work() {
    let config = RustToolchainModernizationConfig::from_toml_path(&support::example_path(
        "soma_rust_toolchain_modernize.toml",
    ))
    .expect("parse sprint76 config");
    assert_eq!(config.target_channel, RustToolchainChannel::Stable);
    assert!(config.require_latest_stable);
    assert!(config.pin_exact_version);
    assert!(config.write_rust_toolchain_toml);
    assert!(!config.allow_nightly);
    let json = serde_json::to_string(&config).expect("serialize config");
    assert!(!json.contains("broker"));
    assert!(!json.contains("account"));
    assert!(!json.contains("live"));

    let bundle = support::run_sprint76_bundle(
        "soma_rust_toolchain_modernize.toml",
        "rust-toolchain-modernization-example",
    );
    assert!(bundle.storage_report.within_budget);
    assert!(bundle.final_summary.contains("toolchain_status"));
}

#[test]
fn sprint76_remote_paths_and_nightly_are_rejected() {
    let mut config = RustToolchainModernizationConfig::default();
    config.output_root = "https://example.com/out".to_string();
    assert!(config.validate().is_err());

    let mut nightly = RustToolchainModernizationConfig::default();
    nightly.allow_nightly = true;
    assert!(nightly.validate().is_err());
}

#[test]
fn rust_toolchain_toml_shape_is_stable_and_stable_only() {
    let path = support::absolutize("rust-toolchain.toml");
    let first = fs::read_to_string(&path).expect("read rust-toolchain.toml");
    let second = fs::read_to_string(&path).expect("read rust-toolchain.toml twice");
    assert_eq!(first, second);
    assert!(first.contains("[toolchain]"));
    assert!(first.contains("channel = \"1.95.0\"") || first.contains("channel = \"stable\""));
    assert!(first.contains("\"rustfmt\""));
    assert!(first.contains("\"clippy\""));
    assert!(first.contains("profile = \"minimal\""));
    assert!(!first.contains("nightly"));
}

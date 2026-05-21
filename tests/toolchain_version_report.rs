#[path = "support/sprint69_support.rs"]
mod support;

use soma_zero::{RustToolchainModernizationRunner, ToolchainVersionReport, ToolchainVersionStatus};

#[test]
fn toolchain_version_report_parses_versions_and_threads_previous_state() {
    let config = support::sprint76_config_from_example(
        "soma_toolchain_version_report.toml",
        "toolchain-version-report",
    );
    let report = RustToolchainModernizationRunner::default()
        .run_toolchain_version_report(&config)
        .expect("toolchain version report");
    assert!(!report.selected_rustc_version.is_empty());
    assert!(!report.selected_cargo_version.is_empty());
    assert!(
        report.selected_channel == "stable"
            || report.selected_channel == report.selected_rustc_version
    );
    assert_eq!(report.previous_rustc_version.as_deref(), Some("1.90.0"));
    assert_eq!(report.previous_cargo_version.as_deref(), Some("1.90.0"));
    assert!(matches!(
        report.toolchain_status,
        ToolchainVersionStatus::LatestStablePinned
            | ToolchainVersionStatus::StablePinned
            | ToolchainVersionStatus::RustupMissing
    ));
}

#[test]
fn toolchain_version_report_is_deterministic() {
    let config = support::sprint76_config_from_example(
        "soma_toolchain_version_report.toml",
        "toolchain-version-report-deterministic",
    );
    let runner = RustToolchainModernizationRunner::default();
    let first = runner
        .run_toolchain_version_report(&config)
        .expect("first report");
    let second = runner
        .run_toolchain_version_report(&config)
        .expect("second report");
    assert_eq!(first, second);
}

#[test]
fn rustup_missing_status_is_supported() {
    let report = ToolchainVersionReport {
        report_id: "rustup-missing".to_string(),
        rustup_available: false,
        previous_rustc_version: Some("1.90.0".to_string()),
        previous_cargo_version: Some("1.90.0".to_string()),
        selected_channel: "stable".to_string(),
        selected_rustc_version: "1.95.0".to_string(),
        selected_cargo_version: "1.95.0".to_string(),
        selected_rustup_version: None,
        rust_toolchain_toml_path: Some("rust-toolchain.toml".to_string()),
        toolchain_status: ToolchainVersionStatus::RustupMissing,
        reason_codes: Vec::new(),
    };
    assert_eq!(
        report.toolchain_status,
        ToolchainVersionStatus::RustupMissing
    );
}

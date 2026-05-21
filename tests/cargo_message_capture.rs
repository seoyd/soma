mod support;

use soma_zero::{
    CargoMessageCaptureStatus, RealWorkspaceTimeoutAttributionConfig,
    Sprint93TimeoutAttributionRunner,
};
use support::{shared_fixture_harness as harness, sprint69_support as sprint};

fn config(name: &str) -> RealWorkspaceTimeoutAttributionConfig {
    sprint::sprint93_config_from_example("soma_cargo_message_capture.toml", name)
}

#[test]
fn cargo_message_capture_counts_targets_and_stays_secret_free() {
    let report = Sprint93TimeoutAttributionRunner::default()
        .run_cargo_message_capture(&config("cargo-message-capture"))
        .expect("report");
    assert_eq!(
        report.status,
        CargoMessageCaptureStatus::CargoMessagesCaptured
    );
    assert_eq!(report.message_count, 7);
    assert_eq!(report.compiler_artifact_count, 6);
    assert_eq!(report.compiler_message_count, 1);
    assert_eq!(report.test_executable_count, 5);
    harness::assert_no_secret_like_values(&serde_json::to_string(&report).expect("json"));
}

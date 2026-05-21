mod support;

use soma_zero::{
    ActiveRustcProcessSnapshotStatus, RealWorkspaceTimeoutAttributionConfig,
    Sprint93TimeoutAttributionRunner,
};
use support::{shared_fixture_harness as harness, sprint69_support as sprint};

fn config(name: &str) -> RealWorkspaceTimeoutAttributionConfig {
    sprint::sprint93_config_from_example("soma_active_rustc_snapshot.toml", name)
}

#[test]
fn active_rustc_snapshot_captures_redacted_local_processes() {
    let report = Sprint93TimeoutAttributionRunner::default()
        .run_active_rustc_snapshot(&config("active-rustc-snapshot"))
        .expect("report");
    assert_eq!(
        report.status,
        ActiveRustcProcessSnapshotStatus::RustcSnapshotsCaptured
    );
    assert_eq!(report.active_process_count_max, Some(2));
    harness::assert_no_secret_like_values(&serde_json::to_string(&report).expect("json"));
}

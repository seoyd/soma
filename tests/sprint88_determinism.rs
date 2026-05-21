mod support;

use support::{shared_fixture_harness as harness, sprint69_support as sprint};

#[test]
fn sprint88_bundle_is_deterministic_for_same_fixture_inputs() {
    let first = sprint::run_sprint88_bundle(
        "soma_sprint88_seven_blocker_recover.toml",
        "sprint88-determinism-first",
    );
    let second = sprint::run_sprint88_bundle(
        "soma_sprint88_seven_blocker_recover.toml",
        "sprint88-determinism-second",
    );
    assert_eq!(
        first.seven_blocker_family_recovery_report,
        second.seven_blocker_family_recovery_report
    );
    assert_eq!(
        first.per_family_compile_probe_reports,
        second.per_family_compile_probe_reports
    );
    assert_eq!(
        first.workspace_gate_recovery_v5,
        second.workspace_gate_recovery_v5
    );
    assert_eq!(
        first.control_tower_seven_blocker_panel,
        second.control_tower_seven_blocker_panel
    );
    harness::assert_deterministic_text(&first.final_summary, &second.final_summary);
}

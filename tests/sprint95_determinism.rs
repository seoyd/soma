mod support;

use support::sprint69_support as sprint;

#[test]
fn sprint95_bundle_is_deterministic() {
    let first = sprint::run_sprint95_bundle(
        "soma_sprint95_committee_cli_safety_recover.toml",
        "sprint95-determinism-a",
    );
    let second = sprint::run_sprint95_bundle(
        "soma_sprint95_committee_cli_safety_recover.toml",
        "sprint95-determinism-b",
    );
    assert_eq!(
        first.committee_cli_safety_reduction_plan,
        second.committee_cli_safety_reduction_plan
    );
    assert_eq!(
        first.committee_cli_safety_reduction_report,
        second.committee_cli_safety_reduction_report
    );
    assert_eq!(
        first.committee_cli_safety_isolation_decision,
        second.committee_cli_safety_isolation_decision
    );
    assert_eq!(
        first.control_tower_committee_cli_safety_recovery_panel,
        second.control_tower_committee_cli_safety_recovery_panel
    );
    assert_eq!(first.final_summary, second.final_summary);
}

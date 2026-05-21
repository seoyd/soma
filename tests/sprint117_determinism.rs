mod support;

use support::sprint117_support::run_sprint117;

#[test]
fn sprint117_bundle_is_deterministic() {
    let first = run_sprint117(
        "soma_sprint117_deferred_real_observation.toml",
        "sprint117-determinism-a",
    );
    let second = run_sprint117(
        "soma_sprint117_deferred_real_observation.toml",
        "sprint117-determinism-b",
    );
    assert_eq!(
        first.sprint116_baseline_truth_import_report,
        second.sprint116_baseline_truth_import_report
    );
    assert_eq!(
        first.deferred_observation_selection_report_v1,
        second.deferred_observation_selection_report_v1
    );
    assert_eq!(
        first.workspace_timeout_evidence_matrix_v3,
        second.workspace_timeout_evidence_matrix_v3
    );
    assert_eq!(
        first.acceptance_truth_gate_v18,
        second.acceptance_truth_gate_v18
    );
    assert_eq!(
        first.control_tower_deferred_observation_execution_panel,
        second.control_tower_deferred_observation_execution_panel
    );
    assert_eq!(first.final_summary, second.final_summary);
    assert_eq!(first.storage_report.file_count, 39);
}

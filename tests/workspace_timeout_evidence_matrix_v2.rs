mod support;

use soma_zero::{
    CommandObservation, Sprint115SummaryFixture, WorkspaceTimeoutEvidenceMatrixV2,
    build_cargo_artifact_ordering_report_v1, build_full_timeout_boundary_report_v1,
    build_no_run_timeout_boundary_report_v1, build_real_cargo_json_observation_attempt_v17,
    build_real_full_workspace_observation_attempt_v17, build_real_no_run_observation_attempt_v17,
    build_timeout_boundary_observation_task_report_v1, build_timeout_cleanup_consistency_report_v1,
    build_workspace_timeout_evidence_matrix_v2,
};
use support::sprint116_support::{read_fixture, run_sprint116};

#[test]
fn workspace_timeout_evidence_matrix_v2_matches_expected() {
    let bundle = run_sprint116(
        "soma_workspace_timeout_evidence_matrix_v2.toml",
        "workspace-timeout-evidence-matrix-v2",
    );
    let expected: WorkspaceTimeoutEvidenceMatrixV2 =
        read_fixture("sprint116_data/timeout_evidence_matrix_expected.json");
    assert_eq!(bundle.workspace_timeout_evidence_matrix_v2, expected);
}

#[test]
fn evidence_matrix_supports_acceptance_only_if_full_finished_and_passed() {
    let summary = Sprint115SummaryFixture::default();
    let no_run = build_real_no_run_observation_attempt_v17(None, Some(420_000));
    let full = build_real_full_workspace_observation_attempt_v17(
        Some(&CommandObservation {
            attempted: true,
            finished: true,
            passed: Some(true),
            duration_ms: Some(1_000),
            timeout_ms: Some(420_000),
            exit_code: Some(0),
            timed_out: false,
            stdout: String::new(),
        }),
        Some(420_000),
    );
    let cargo_json = build_real_cargo_json_observation_attempt_v17(None, Some(420_000));
    let boundary = build_no_run_timeout_boundary_report_v1(&summary, &no_run, Some(420_000));
    let cleanup = build_timeout_cleanup_consistency_report_v1(&summary);
    let ordering = build_cargo_artifact_ordering_report_v1(&cargo_json);
    let matrix = build_workspace_timeout_evidence_matrix_v2(
        &summary,
        &no_run,
        &full,
        &cargo_json,
        &boundary,
        &cleanup,
        &ordering,
    );
    assert!(
        matrix
            .evidence_rows
            .iter()
            .any(|row| row.row_id == "FullObservation" && row.supports_acceptance)
    );
}

#[test]
fn carried_forward_fixture_data_is_not_marked_as_actual_observation() {
    let summary = Sprint115SummaryFixture::default();
    let no_run = build_real_no_run_observation_attempt_v17(None, Some(420_000));
    let full = build_real_full_workspace_observation_attempt_v17(None, Some(420_000));
    let no_run_boundary = build_no_run_timeout_boundary_report_v1(&summary, &no_run, Some(420_000));
    let full_boundary = build_full_timeout_boundary_report_v1(&summary, &full, Some(420_000));
    let boundary_task =
        build_timeout_boundary_observation_task_report_v1(&no_run_boundary, &full_boundary);
    let cargo_json = build_real_cargo_json_observation_attempt_v17(None, Some(420_000));
    let ordering = build_cargo_artifact_ordering_report_v1(&cargo_json);

    assert!(!no_run_boundary.actual_observation);
    assert!(!full_boundary.actual_observation);
    assert_eq!(boundary_task.task_status, "TimeoutBoundaryCarriedForward");
    assert!(ordering.observed_artifact_order.is_empty());
    assert_eq!(ordering.status, "CargoArtifactOrderingDeferred");
}

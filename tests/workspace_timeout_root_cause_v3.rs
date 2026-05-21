mod support;

use serde_json::Value;
use soma_zero::{
    RealCargoFullObservationV1, RealCargoJsonProgressObservationV1, RealCargoNoRunObservationV1,
    Sprint112SummaryFixture, SuspectTargetFamilyRegistryV1,
    SuspectTargetFixtureRenderCliSplitReportV1, SuspectTargetLinkMacroSplitReportV1,
    build_workspace_timeout_root_cause_report_v3,
};
use support::sprint113_support::{read_fixture, run_sprint113};

#[test]
fn workspace_timeout_root_cause_keeps_observed_and_inferred_split() {
    let bundle = run_sprint113(
        "soma_workspace_timeout_root_cause_v3.toml",
        "workspace-timeout-root-cause-v3",
    );
    let expected: Value = read_fixture("sprint113_data/root_cause_v3_expected.json");
    assert_eq!(
        bundle
            .workspace_timeout_root_cause_report_v3
            .root_cause_status,
        expected["root_cause_status"].as_str().unwrap()
    );
    assert!(
        bundle
            .workspace_timeout_root_cause_report_v3
            .observed_evidence
            .iter()
            .any(|item| item.contains("timeout"))
    );
    assert!(
        bundle
            .workspace_timeout_root_cause_report_v3
            .inferred_evidence
            .contains(&"MacroExpansionCost".to_string())
    );
    assert!(!bundle.root_cause_evidence_upgrade_report_v1.upgraded);

    let summary = Sprint112SummaryFixture::default();
    let root = build_workspace_timeout_root_cause_report_v3(
        &summary,
        &RealCargoJsonProgressObservationV1 {
            observation_id: "id".to_string(),
            attempted: true,
            command: "cmd".to_string(),
            finished: false,
            timed_out: true,
            message_count: 2,
            artifact_count: 1,
            compiler_message_count: 0,
            parsed_json_message_count: 2,
            parse_error_count: 0,
            last_seen_targets: vec!["tests/workspace_timeout_root_cause.rs".to_string()],
            last_seen_artifacts: vec![],
            stalled_candidates: vec!["tests/workspace_timeout_root_cause.rs".to_string()],
            observation_status: "CargoJsonTimedOut".to_string(),
            reason_codes: vec![],
        },
        &RealCargoNoRunObservationV1 {
            observation_id: "id".to_string(),
            attempted: true,
            command: "cmd".to_string(),
            started: true,
            finished: false,
            passed: None,
            duration_ms: None,
            timeout_ms: Some(1),
            exit_code: Some(124),
            timed_out: true,
            last_seen_target: None,
            child_process_cleanup_verified: true,
            observation_status: "RealNoRunTimedOut".to_string(),
            reason_codes: vec![],
        },
        &RealCargoFullObservationV1 {
            observation_id: "id".to_string(),
            attempted: false,
            command: "cmd".to_string(),
            started: false,
            finished: false,
            passed: None,
            duration_ms: None,
            timeout_ms: None,
            exit_code: None,
            timed_out: false,
            last_seen_target: None,
            child_process_cleanup_verified: true,
            observation_status: "RealFullNotRun".to_string(),
            reason_codes: vec![],
        },
        &SuspectTargetFamilyRegistryV1 {
            registry_id: "id".to_string(),
            suspect_targets: vec!["tests/workspace_timeout_root_cause.rs".to_string()],
            suspect_families: vec!["MacroExpansionCost".to_string()],
            already_retired_targets_excluded: true,
            sentinel_targets_excluded: true,
            registry_status: "SuspectRegistryReady".to_string(),
            reason_codes: vec![],
        },
        &SuspectTargetLinkMacroSplitReportV1 {
            report_id: "id".to_string(),
            link_heavy_suspect: vec![],
            macro_heavy_suspect: vec!["tests/workspace_timeout_root_cause.rs".to_string()],
            observed_labels: vec!["tests/workspace_timeout_root_cause.rs".to_string()],
            inferred_labels: vec!["MacroExpansionCost".to_string()],
            split_status: "ready".to_string(),
            reason_codes: vec![],
        },
        &SuspectTargetFixtureRenderCliSplitReportV1 {
            report_id: "id".to_string(),
            fixture_pressure: vec![],
            render_pressure: vec!["tests/workspace_timeout_root_cause.rs".to_string()],
            cli_pressure: vec![],
            observed_labels: vec![],
            inferred_labels: vec!["ArtifactRenderFanout".to_string()],
            split_status: "ready".to_string(),
            reason_codes: vec![],
        },
    );
    assert_eq!(root.root_cause_status, "TimeoutRootCauseIsolated");
    assert_eq!(root.root_cause_confidence, "Strong");
}

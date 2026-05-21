mod support;

use serde_json::Value;
use soma_zero::{
    FifthPatchAssertionMigrationFeasibilityReportV1,
    FifthPatchEquivalentCoverageFeasibilityReportV1, FifthPatchSentinelSafetyFeasibilityReportV1,
    NoHiddenSkipContinuityCheckV3, RemainingSafeCandidatePoolReportV3, Sprint112SummaryFixture,
    WorkspaceTimeoutRootCauseReportV3, build_fifth_patch_decision_gate_v3,
};
use support::sprint113_support::{read_fixture, run_sprint113};

#[test]
fn fifth_patch_gate_stays_no_apply_and_next_sprint_only() {
    let bundle = run_sprint113(
        "soma_fifth_patch_decision_gate_v3.toml",
        "fifth-patch-decision-gate-v3",
    );
    let expected: Value = read_fixture("sprint113_data/fifth_patch_gate_v3_expected.json");
    assert_eq!(
        bundle.fifth_patch_decision_gate_v3.gate_status,
        expected["gate_status"].as_str().unwrap()
    );
    assert!(
        !bundle
            .fifth_patch_decision_gate_v3
            .fifth_patch_applied_this_sprint
    );
    assert!(
        !bundle
            .fifth_patch_decision_gate_v3
            .fifth_patch_allowed_for_next_sprint
    );
    assert_eq!(
        bundle
            .fifth_patch_no_apply_guarantee_report_v2
            .guarantee_status,
        "FifthPatchNoApplyGuaranteed"
    );

    let summary = Sprint112SummaryFixture::default();
    let allowed = build_fifth_patch_decision_gate_v3(
        &summary,
        &WorkspaceTimeoutRootCauseReportV3 {
            report_id: "id".to_string(),
            previous_status: "old".to_string(),
            new_real_observation_refs: vec!["real".to_string()],
            observed_evidence: vec!["a".to_string(); 5],
            inferred_evidence: vec![],
            suspect_target_evidence: vec!["t1".to_string(), "t2".to_string()],
            root_cause_confidence: "Strong".to_string(),
            root_cause_status: "TimeoutRootCauseIsolated".to_string(),
            reason_codes: vec![],
        },
        &RemainingSafeCandidatePoolReportV3 {
            report_id: "id".to_string(),
            previous_candidate_pool: vec![],
            updated_evidence: vec![],
            candidate_statuses: std::collections::BTreeMap::from([(
                "target".to_string(),
                "LowRiskCandidate".to_string(),
            )]),
            assertion_migration_feasible_candidates: vec!["target".to_string()],
            equivalent_coverage_feasible_candidates: vec!["target".to_string()],
            sentinel_exclusions: vec![],
            status: "CandidatePoolReadyWithWarnings".to_string(),
            reason_codes: vec![],
        },
        &FifthPatchAssertionMigrationFeasibilityReportV1 {
            report_id: "id".to_string(),
            candidate: Some("target".to_string()),
            assertion_moves_required: 0,
            feasibility: "Feasible".to_string(),
            blockers: vec![],
            status: "AssertionMigrationFeasible".to_string(),
            reason_codes: vec![],
        },
        &FifthPatchEquivalentCoverageFeasibilityReportV1 {
            report_id: "id".to_string(),
            equivalent_coverage_possible: true,
            destination_target: Some("target".to_string()),
            coverage_gaps: vec![],
            status: "EquivalentCoverageFeasible".to_string(),
            reason_codes: vec![],
        },
        &FifthPatchSentinelSafetyFeasibilityReportV1 {
            report_id: "id".to_string(),
            sentinel_risk: "Low".to_string(),
            workspace_cli_safety_risk: "Low".to_string(),
            determinism_risk: "Low".to_string(),
            paper_lifecycle_safety_risk: "Low".to_string(),
            status: "SentinelSafetyFeasible".to_string(),
            reason_codes: vec![],
        },
        &NoHiddenSkipContinuityCheckV3 {
            report_id: "id".to_string(),
            hidden_skip_indicators: vec![],
            continuity_status: "NoHiddenSkipContinuityReady".to_string(),
            reason_codes: vec![],
        },
    );
    assert_eq!(allowed.gate_status, "FifthPatchAllowedForNextSprint");
    assert!(allowed.fifth_patch_allowed_for_next_sprint);
    assert!(!allowed.fifth_patch_applied_this_sprint);
}

mod support;

use soma_zero::{
    AssertionMigrationFeasibilityDrilldownReportV1, EquivalentCoverageFeasibilityDrilldownReportV1,
    FifthPatchCandidateDecisionMatrixV1, FifthPatchCandidateDecisionRowV1,
    IntegrationFanoutNarrowingReportV1, LinkTimeNarrowingReportV1, MacroExpansionNarrowingReportV1,
    NoHiddenSkipRiskPreviewReportV1, SentinelSafetyImpactPreviewReportV1, Sprint113SummaryFixture,
    build_fifth_patch_decision_gate_v4,
};
use support::sprint114_support::run_sprint114;

#[test]
fn fifth_patch_gate_handles_blocked_ready_and_safety_cases() {
    let bundle = run_sprint114(
        "soma_fifth_patch_decision_gate_v4.toml",
        "fifth-patch-decision-gate-v4",
    );
    assert_eq!(
        bundle.fifth_patch_decision_gate_v4.gate_status,
        "FifthPatchStillBlocked"
    );
    assert!(
        !bundle
            .fifth_patch_decision_gate_v4
            .fifth_patch_ready_for_next_sprint
    );
    assert!(
        !bundle
            .fifth_patch_decision_gate_v4
            .fifth_patch_applied_this_sprint
    );

    let summary = Sprint113SummaryFixture {
        mixed_family_evidence_narrowed_enough: true,
        ..Sprint113SummaryFixture::default()
    };
    let integration = IntegrationFanoutNarrowingReportV1 {
        report_id: "i".to_string(),
        suspect_integration_targets: vec![],
        fanout_cluster_count: 1,
        isolated_integration_fanout: vec![],
        still_mixed_integration_fanout: vec![],
        observed_evidence: vec!["o".to_string()],
        inferred_evidence: vec!["i".to_string()],
        status: "IntegrationFanoutNarrowed".to_string(),
        reason_codes: vec![],
    };
    let link = LinkTimeNarrowingReportV1 {
        report_id: "l".to_string(),
        link_heavy_target_candidates: vec![],
        observed_evidence: vec!["o".to_string()],
        inferred_evidence: vec!["i".to_string()],
        status: "LinkTimeNarrowed".to_string(),
        reason_codes: vec![],
    };
    let mac = MacroExpansionNarrowingReportV1 {
        report_id: "m".to_string(),
        macro_heavy_target_candidates: vec![],
        observed_evidence: vec!["o".to_string()],
        inferred_evidence: vec!["i".to_string()],
        status: "MacroExpansionNarrowed".to_string(),
        reason_codes: vec![],
    };
    let assertion = AssertionMigrationFeasibilityDrilldownReportV1 {
        report_id: "a".to_string(),
        candidate_target: "t".to_string(),
        assertions_to_move: vec![],
        destination_candidates: vec![],
        blockers: vec![],
        feasible: true,
        feasibility_status: "AssertionMigrationFeasible".to_string(),
        reason_codes: vec![],
    };
    let equivalent = EquivalentCoverageFeasibilityDrilldownReportV1 {
        report_id: "e".to_string(),
        equivalent_coverage_destination: Some("dest".to_string()),
        coverage_proof_refs: vec![],
        coverage_gaps: vec![],
        feasible: true,
        status: "EquivalentCoverageFeasible".to_string(),
        reason_codes: vec![],
    };
    let sentinel = SentinelSafetyImpactPreviewReportV1 {
        report_id: "s".to_string(),
        sentinel_impact: "Preserved".to_string(),
        isolated_sentinels_affected: vec![],
        sentinel_safety_preserved: true,
        status: "SentinelSafetyPreviewReady".to_string(),
        reason_codes: vec![],
    };
    let hidden = NoHiddenSkipRiskPreviewReportV1 {
        report_id: "h".to_string(),
        skip_risk_indicators: vec![],
        hidden_skip_risk: false,
        status: "NoHiddenSkipRiskPreviewReady".to_string(),
        reason_codes: vec![],
    };
    let matrix = FifthPatchCandidateDecisionMatrixV1 {
        report_id: "x".to_string(),
        candidate_rows: vec![FifthPatchCandidateDecisionRowV1 {
            candidate_target: "t".to_string(),
            assertion_migration_feasible: true,
            equivalent_coverage_feasible: true,
            sentinel_safety_preserved: true,
            no_hidden_skip_risk: true,
            mixed_family_relevance: "High".to_string(),
            decision_recommendation: "ReadyForNextSprint".to_string(),
        }],
        matrix_status: "ready".to_string(),
        reason_codes: vec![],
    };
    let ready = build_fifth_patch_decision_gate_v4(
        &summary,
        &integration,
        &link,
        &mac,
        &assertion,
        &equivalent,
        &sentinel,
        &hidden,
        &matrix,
    );
    assert_eq!(ready.gate_status, "FifthPatchReadyForNextSprint");

    let blocked = build_fifth_patch_decision_gate_v4(
        &summary,
        &integration,
        &link,
        &mac,
        &assertion,
        &equivalent,
        &SentinelSafetyImpactPreviewReportV1 {
            sentinel_safety_preserved: false,
            status: "SentinelSafetyPreviewBlocked".to_string(),
            ..sentinel.clone()
        },
        &hidden,
        &matrix,
    );
    assert_eq!(blocked.gate_status, "FifthPatchBlockedBySafety");
}

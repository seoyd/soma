use soma_zero::{
    AcceptanceTruthGateV12, EquivalentCoverageContinuityCheckV1,
    FifthPatchCandidatePreselectionReport, NoHiddenSkipContinuityCheckV1,
    RemainingSafeConsolidationCandidatePoolReport, SafetySentinelContinuityCheckV1,
    WorkspaceTimeoutRootCauseReport, build_fifth_patch_decision_gate,
};

fn base_pool() -> RemainingSafeConsolidationCandidatePoolReport {
    RemainingSafeConsolidationCandidatePoolReport {
        report_id: "pool".to_string(),
        candidate_pool: vec!["tests/shared_fixture_harness_application_v1.rs".to_string()],
        low_risk_candidates: vec!["tests/shared_fixture_harness_application_v1.rs".to_string()],
        medium_risk_candidates: Vec::new(),
        high_risk_candidates: Vec::new(),
        sentinel_candidates_excluded: vec!["tests/committee_cli_safety.rs".to_string()],
        candidates_with_equivalent_coverage_feasible: vec![
            "tests/shared_fixture_harness_application_v1.rs".to_string(),
        ],
        candidates_needing_more_evidence: Vec::new(),
        pool_status: "CandidatePoolReadyWithWarnings".to_string(),
        reason_codes: Vec::new(),
    }
}

fn base_preselection(candidate: &str) -> FifthPatchCandidatePreselectionReport {
    FifthPatchCandidatePreselectionReport {
        report_id: "pre".to_string(),
        preselected_candidate: Some(candidate.to_string()),
        candidate_reason: "candidate".to_string(),
        expected_assertion_moves: vec!["move".to_string()],
        expected_equivalent_coverage_refs: vec!["eq".to_string()],
        risk_preview: "risk".to_string(),
        preselection_status: "FifthPatchCandidatePreselected".to_string(),
        reason_codes: Vec::new(),
    }
}

fn base_equivalent(gaps: usize) -> EquivalentCoverageContinuityCheckV1 {
    EquivalentCoverageContinuityCheckV1 {
        report_id: "eq".to_string(),
        previous_equivalent_coverage_proofs_loaded: 4,
        coverage_gaps: gaps,
        continuity_status: "EquivalentCoverageContinuityReady".to_string(),
        reason_codes: Vec::new(),
    }
}

fn base_sentinel() -> SafetySentinelContinuityCheckV1 {
    SafetySentinelContinuityCheckV1 {
        report_id: "sentinel".to_string(),
        sentinels_preserved_across_sprints: vec!["CommitteeCliSafety".to_string()],
        continuity_status: "SafetySentinelContinuityReady".to_string(),
        reason_codes: Vec::new(),
    }
}

fn base_skip(indicators: Vec<String>) -> NoHiddenSkipContinuityCheckV1 {
    NoHiddenSkipContinuityCheckV1 {
        report_id: "skip".to_string(),
        hidden_skip_indicators: indicators,
        skip_status: "NoHiddenSkipContinuityReady".to_string(),
        reason_codes: Vec::new(),
    }
}

fn base_root() -> WorkspaceTimeoutRootCauseReport {
    WorkspaceTimeoutRootCauseReport {
        report_id: "root".to_string(),
        no_run_timeout_observed: true,
        full_timeout_observed: true,
        cargo_json_progress_available: true,
        last_seen_targets: vec!["tests/shared_fixture_harness_application_v1.rs".to_string()],
        last_seen_artifacts: vec!["artifact".to_string()],
        suspected_root_causes: vec!["IntegrationTestBinaryFanout".to_string()],
        evidence_strength: "Moderate".to_string(),
        root_cause_status: "TimeoutRootCausePartiallyIsolated".to_string(),
        reason_codes: Vec::new(),
    }
}

fn base_truth() -> AcceptanceTruthGateV12 {
    AcceptanceTruthGateV12 {
        gate_id: "truth".to_string(),
        focused_truth_status: "SupportingOnly".to_string(),
        cli_truth_status: "SupportingOnly".to_string(),
        cargo_build_truth_status: "SupportingOnly".to_string(),
        no_run_truth_status: "SupportingOnly".to_string(),
        full_workspace_truth_status: "SupportingOnly".to_string(),
        truth_status: "AcceptanceTruthReadyWithWarnings".to_string(),
        reason_codes: Vec::new(),
    }
}

#[test]
fn fifth_patch_decision_gate_blocks_required_failure_modes_and_allows_with_warnings() {
    let allowed = build_fifth_patch_decision_gate(
        &base_pool(),
        &base_preselection("tests/shared_fixture_harness_application_v1.rs"),
        &base_equivalent(0),
        &base_sentinel(),
        &base_skip(Vec::new()),
        &base_root(),
        &base_truth(),
    );
    assert!(allowed.fifth_patch_allowed);
    assert_eq!(allowed.gate_status, "FifthPatchAllowedWithWarnings");

    let no_equivalent = build_fifth_patch_decision_gate(
        &base_pool(),
        &base_preselection("tests/shared_fixture_harness_application_v1.rs"),
        &base_equivalent(1),
        &base_sentinel(),
        &base_skip(Vec::new()),
        &base_root(),
        &base_truth(),
    );
    assert!(!no_equivalent.fifth_patch_allowed);

    let high_risk_sentinel = build_fifth_patch_decision_gate(
        &base_pool(),
        &base_preselection("tests/committee_cli_safety.rs"),
        &base_equivalent(0),
        &base_sentinel(),
        &base_skip(Vec::new()),
        &base_root(),
        &base_truth(),
    );
    assert!(!high_risk_sentinel.fifth_patch_allowed);
    assert_eq!(high_risk_sentinel.gate_status, "FifthPatchBlockedBySafety");

    let hidden_skip = build_fifth_patch_decision_gate(
        &base_pool(),
        &base_preselection("tests/shared_fixture_harness_application_v1.rs"),
        &base_equivalent(0),
        &base_sentinel(),
        &base_skip(vec!["cfg(skip)".to_string()]),
        &base_root(),
        &base_truth(),
    );
    assert!(!hidden_skip.fifth_patch_allowed);

    let already_retired_or_not_in_pool = build_fifth_patch_decision_gate(
        &base_pool(),
        &base_preselection("tests/shared_toml_builder_application_v1.rs"),
        &base_equivalent(0),
        &base_sentinel(),
        &base_skip(Vec::new()),
        &base_root(),
        &base_truth(),
    );
    assert!(!already_retired_or_not_in_pool.fifth_patch_allowed);
    assert_eq!(
        already_retired_or_not_in_pool.gate_status,
        "FifthPatchBlockedPendingEvidence"
    );
}

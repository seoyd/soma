use soma_zero::{
    EquivalentCoverageContinuityCheckV2, NoHiddenSkipContinuityCheckV2,
    RemainingSafeCandidatePoolReportV2, SafetySentinelContinuityCheckV2,
    WorkspaceTimeoutRootCauseReportV2, build_fifth_patch_decision_gate_v2,
};

fn pool() -> RemainingSafeCandidatePoolReportV2 {
    RemainingSafeCandidatePoolReportV2 {
        report_id: "pool".to_string(),
        previous_candidate_pool: vec!["tests/workspace_timeout_root_cause.rs".to_string()],
        new_evidence: vec!["diagnostic".to_string()],
        candidate_statuses: std::collections::BTreeMap::from([(
            "tests/workspace_timeout_root_cause.rs".to_string(),
            "LowRiskCandidate".to_string(),
        )]),
        sentinel_exclusions: vec!["tests/committee_cli_safety.rs".to_string()],
        status: "CandidatePoolReadyWithWarnings".to_string(),
        reason_codes: Vec::new(),
    }
}

fn evidence_only_pool() -> RemainingSafeCandidatePoolReportV2 {
    RemainingSafeCandidatePoolReportV2 {
        report_id: "pool".to_string(),
        previous_candidate_pool: vec!["tests/workspace_timeout_root_cause.rs".to_string()],
        new_evidence: vec!["diagnostic".to_string()],
        candidate_statuses: std::collections::BTreeMap::from([(
            "tests/workspace_timeout_root_cause.rs".to_string(),
            "NeedsMoreEvidence".to_string(),
        )]),
        sentinel_exclusions: Vec::new(),
        status: "NoSafeCandidatePool".to_string(),
        reason_codes: Vec::new(),
    }
}

fn root(status: &str) -> WorkspaceTimeoutRootCauseReportV2 {
    WorkspaceTimeoutRootCauseReportV2 {
        report_id: "root".to_string(),
        root_cause_categories: vec!["FixtureSetupFanout".to_string()],
        observed_evidence: vec!["observed".to_string()],
        inferred_evidence: vec!["inferred".to_string()],
        confidence: if status == "TimeoutRootCauseIsolated" {
            "Strong"
        } else {
            "Moderate"
        }
        .to_string(),
        status: status.to_string(),
        reason_codes: Vec::new(),
    }
}

fn equivalent(ok: bool) -> EquivalentCoverageContinuityCheckV2 {
    EquivalentCoverageContinuityCheckV2 {
        report_id: "eq".to_string(),
        coverage_gap_count: if ok { 0 } else { 1 },
        equivalent_coverage_feasible: ok,
        continuity_status: if ok {
            "EquivalentCoverageContinuityReady"
        } else {
            "EquivalentCoverageContinuityBlocked"
        }
        .to_string(),
        reason_codes: Vec::new(),
    }
}

fn sentinel(ok: bool) -> SafetySentinelContinuityCheckV2 {
    SafetySentinelContinuityCheckV2 {
        report_id: "sentinel".to_string(),
        sentinels_preserved: vec!["CommitteeCliSafety".to_string()],
        sentinel_uncertainties: if ok {
            Vec::new()
        } else {
            vec!["uncertain".to_string()]
        },
        continuity_status: if ok {
            "SafetySentinelContinuityReady"
        } else {
            "SafetySentinelContinuityBlocked"
        }
        .to_string(),
        reason_codes: Vec::new(),
    }
}

fn skip(ok: bool) -> NoHiddenSkipContinuityCheckV2 {
    NoHiddenSkipContinuityCheckV2 {
        report_id: "skip".to_string(),
        hidden_skip_indicators: if ok {
            Vec::new()
        } else {
            vec!["cfg(skip)".to_string()]
        },
        continuity_status: if ok {
            "NoHiddenSkipContinuityReady"
        } else {
            "NoHiddenSkipContinuityBlocked"
        }
        .to_string(),
        reason_codes: Vec::new(),
    }
}

#[test]
fn fifth_patch_decision_gate_v2_respects_safety_and_next_sprint_only_rule() {
    let blocked = build_fifth_patch_decision_gate_v2(
        &pool(),
        &root("TimeoutRootCausePartiallyIsolated"),
        &equivalent(true),
        &sentinel(true),
        &skip(true),
        false,
        "FifthPatchBlockedPendingEvidence".to_string(),
    );
    assert!(!blocked.fifth_patch_allowed_for_next_sprint);
    assert_eq!(blocked.gate_status, "FifthPatchStillBlocked");
    assert!(!blocked.fifth_patch_applied_this_sprint);

    let safety_blocked = build_fifth_patch_decision_gate_v2(
        &pool(),
        &root("TimeoutRootCauseIsolated"),
        &equivalent(true),
        &sentinel(false),
        &skip(true),
        true,
        "FifthPatchBlockedPendingEvidence".to_string(),
    );
    assert_eq!(safety_blocked.gate_status, "FifthPatchBlockedBySafety");

    let allowed = build_fifth_patch_decision_gate_v2(
        &pool(),
        &root("TimeoutRootCauseIsolated"),
        &equivalent(true),
        &sentinel(true),
        &skip(true),
        true,
        "FifthPatchBlockedPendingEvidence".to_string(),
    );
    assert!(allowed.fifth_patch_allowed_for_next_sprint);
    assert_eq!(allowed.gate_status, "FifthPatchAllowedForNextSprint");
    assert!(!allowed.fifth_patch_applied_this_sprint);

    let no_safe_candidate = build_fifth_patch_decision_gate_v2(
        &evidence_only_pool(),
        &root("TimeoutRootCauseIsolated"),
        &equivalent(true),
        &sentinel(true),
        &skip(true),
        true,
        "FifthPatchBlockedPendingEvidence".to_string(),
    );
    assert!(!no_safe_candidate.fifth_patch_allowed_for_next_sprint);
    assert_eq!(no_safe_candidate.gate_status, "FifthPatchStillBlocked");
}

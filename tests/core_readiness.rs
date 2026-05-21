use soma_zero::{
    AuditSummary, CoreContractRegistry, CoreNextRecommendation, CorePerformanceBudget,
    CoreReadinessStatus, DeterminismCheck, DeterminismInputFingerprint,
    DeterminismOutputFingerprint, LiveSafetyStatus, ReasonCode, RuntimeMode, RuntimeStage,
    RuntimeState, RuntimeStateReport, audit_reason_codes, build_live_safety_report,
    build_risk_invariant_report, evaluate_core_readiness, measure_performance_budget,
};

fn runtime_report() -> soma_zero::RuntimeStateReport {
    RuntimeStateReport::from_state(RuntimeState::new(RuntimeMode::Research))
}

fn contract_report() -> soma_zero::CoreContractRegistryReport {
    CoreContractRegistry::default().report()
}

fn determinism_report(deterministic: bool) -> DeterminismCheck {
    let input = DeterminismInputFingerprint::new("fixture", "config", None, None, None);
    let left = DeterminismOutputFingerprint::new("report", 1, &[], 0, 0);
    let right = if deterministic {
        left.clone()
    } else {
        DeterminismOutputFingerprint::new("report-other", 1, &[], 0, 0)
    };
    DeterminismCheck::compare(input, &left, &right)
}

fn audit_summary() -> AuditSummary {
    AuditSummary {
        total_records: 1,
        stages_seen: vec![RuntimeStage::Init],
        decisions_seen: 1,
        risk_decisions_seen: 1,
        failures_seen: 0,
        missing_reason_code_count: 0,
        fingerprint: "fp".to_string(),
        reason_codes: vec![],
    }
}

fn complete_reason_audit() -> soma_zero::ReasonCodeAudit {
    audit_reason_codes(
        &[
            ReasonCode::MissingFile,
            ReasonCode::RemotePathRejected,
            ReasonCode::MissingAuth,
            ReasonCode::DataQualityTooLow,
            ReasonCode::SchemaMismatch,
            ReasonCode::InvalidPrediction,
            ReasonCode::BudgetExceeded,
            ReasonCode::PreflightFailed,
            ReasonCode::RiskDenied,
            ReasonCode::NoTradeDefault,
            ReasonCode::LiveModeDisabled,
        ],
        &[],
        None,
    )
}

fn performance_report() -> soma_zero::CorePerformanceBudgetReport {
    measure_performance_budget(&CorePerformanceBudget::default(), 0, 0, 0, 0, 0, 0, 0, &[])
}

#[test]
fn risk_invariant_failure_blocks_core_readiness() {
    let mut risk = build_risk_invariant_report();
    risk.default_deny_passed = false;

    let report = evaluate_core_readiness(
        runtime_report(),
        contract_report(),
        determinism_report(true),
        complete_reason_audit(),
        audit_summary(),
        risk,
        build_live_safety_report(&["core-check".to_string()], false),
        performance_report(),
        false,
        true,
        false,
    );

    assert_eq!(
        report.final_status,
        CoreReadinessStatus::NotReadyDueToRiskInvariantFailure
    );
}

#[test]
fn nondeterminism_blocks_core_readiness() {
    let report = evaluate_core_readiness(
        runtime_report(),
        contract_report(),
        determinism_report(false),
        complete_reason_audit(),
        audit_summary(),
        build_risk_invariant_report(),
        build_live_safety_report(&["core-check".to_string()], false),
        performance_report(),
        false,
        true,
        false,
    );

    assert_eq!(
        report.final_status,
        CoreReadinessStatus::NotReadyDueToNondeterminism
    );
}

#[test]
fn live_safety_gap_blocks_core_readiness() {
    let mut live = build_live_safety_report(&["live-order".to_string()], false);
    live.status = LiveSafetyStatus::UnsafePathDetected;

    let report = evaluate_core_readiness(
        runtime_report(),
        contract_report(),
        determinism_report(true),
        complete_reason_audit(),
        audit_summary(),
        build_risk_invariant_report(),
        live,
        performance_report(),
        false,
        true,
        false,
    );

    assert_eq!(
        report.final_status,
        CoreReadinessStatus::NotReadyDueToLiveSafetyGap
    );
}

#[test]
fn contract_drift_blocks_core_readiness() {
    let mut report = contract_report();
    report.checks[0].compatible = false;

    let readiness = evaluate_core_readiness(
        runtime_report(),
        report,
        determinism_report(true),
        complete_reason_audit(),
        audit_summary(),
        build_risk_invariant_report(),
        build_live_safety_report(&["core-check".to_string()], false),
        performance_report(),
        false,
        true,
        false,
    );

    assert_eq!(
        readiness.final_status,
        CoreReadinessStatus::NotReadyDueToContractDrift
    );
}

#[test]
fn all_core_checks_pass_to_conservative_more_official_evidence_status() {
    let readiness = evaluate_core_readiness(
        runtime_report(),
        contract_report(),
        determinism_report(true),
        complete_reason_audit(),
        audit_summary(),
        build_risk_invariant_report(),
        build_live_safety_report(&["core-check".to_string()], false),
        performance_report(),
        false,
        true,
        false,
    );

    assert_eq!(
        readiness.final_status,
        CoreReadinessStatus::ReadyForMoreOfficialEvidence
    );
    assert_eq!(
        readiness.next_recommendation,
        CoreNextRecommendation::MoreOfficialEvidence
    );
}

#[test]
fn readiness_rendering_is_deterministic() {
    let left = evaluate_core_readiness(
        runtime_report(),
        contract_report(),
        determinism_report(true),
        complete_reason_audit(),
        audit_summary(),
        build_risk_invariant_report(),
        build_live_safety_report(&["core-check".to_string()], false),
        performance_report(),
        true,
        true,
        true,
    );
    let right = evaluate_core_readiness(
        runtime_report(),
        contract_report(),
        determinism_report(true),
        complete_reason_audit(),
        audit_summary(),
        build_risk_invariant_report(),
        build_live_safety_report(&["core-check".to_string()], false),
        performance_report(),
        true,
        true,
        true,
    );

    assert_eq!(left.to_text(), right.to_text());
}

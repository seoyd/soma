mod support;

use soma_zero::Sprint105VerificationPatchClosureRunner;
use support::sprint105_support::run_sprint105;
use support::sprint105_support::write_support_json;

#[test]
fn risk_governor_required_transitions_are_complete() {
    let bundle = run_sprint105(
        "soma_risk_required_transition_audit.toml",
        "risk_required_transition_audit",
    );
    let report = &bundle.risk_governor_required_transition_audit_report;
    assert!(report.paper_approved_requires_risk);
    assert!(report.paper_rejected_requires_risk);
    assert!(report.risk_denied_requires_risk);
    assert!(report.no_trade_requires_risk);
    assert!(report.cooldown_requires_risk);
    assert!(!report.live_transition_present);
    assert_eq!(report.bypass_transition_count, 0);
}

#[test]
fn risk_governor_required_transitions_detect_missing_cooldown_risk_review() {
    let lifecycle = write_support_json(
        "risk_required_transition_missing_cooldown",
        "lifecycle.json",
        &serde_json::json!({
            "machine_id": "paper-candidate-lifecycle-state-machine",
            "allowed_transitions": [
                "DebateOpen->PaperApproved",
                "PaperApproved->Cooldown",
                "NoTrade->Cooldown"
            ],
            "forbidden_transitions": [],
            "risk_governor_required_transitions": [
                "DebateOpen->PaperApproved",
                "DebateOpen->PaperRejected",
                "DebateOpen->RiskDenied",
                "DebateOpen->NoTrade"
            ],
            "broker_execution_allowed": false,
            "live_execution_allowed": false,
            "state_machine_status": "PaperLifecycleReadyWithWarnings",
            "reason_codes": []
        }),
    );
    let risk_batch = write_support_json(
        "risk_required_transition_missing_cooldown_batch",
        "risk_batch.json",
        &serde_json::json!({
            "report_id": "risk-governor-batch-veto-report",
            "batch_count": 1,
            "approved_paper_only_count": 1,
            "no_trade_count": 0,
            "risk_denied_count": 0,
            "cooldown_count": 1,
            "need_more_evidence_count": 0,
            "bypass_attempt_count": 0,
            "broker_execution_allowed_count": 0,
            "live_execution_allowed_count": 0,
            "veto_status": "RiskGovernorBatchVetoReadyWithWarnings",
            "reason_codes": []
        }),
    );
    let mut config = soma_zero::Sprint105VerificationPatchClosureConfig::default();
    config.paper_lifecycle_paths = Some(vec![lifecycle]);
    config.risk_governor_batch_paths = Some(vec![risk_batch]);
    let bundle = Sprint105VerificationPatchClosureRunner::default()
        .run(&config)
        .expect("run");
    assert!(
        !bundle
            .risk_governor_required_transition_audit_report
            .cooldown_requires_risk
    );
    assert_eq!(
        bundle
            .risk_governor_required_transition_audit_report
            .audit_status,
        "RiskGovernorRequiredTransitionsIncomplete"
    );
}

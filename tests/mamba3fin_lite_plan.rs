mod common;

use std::collections::BTreeMap;
use std::fs;

use soma_zero::{
    Mamba3FinLitePrototypeBackend, Mamba3FinLitePrototypePlanConfig,
    Mamba3FinLitePrototypePlanRunner, Mamba3ReadinessAuditV2, Mamba3ReadinessRecommendation,
    Mamba3ReadinessState, ModelEscalationCandidate, ModelEscalationDecisionStatus,
    ModelEscalationDecisionV2, SequenceDatasetReadinessReport, SequenceDatasetReadinessStatus,
};

fn sequence_report() -> SequenceDatasetReadinessReport {
    SequenceDatasetReadinessReport {
        readiness_id: "sequence".to_string(),
        row_count: 4096,
        official_row_count: 3584,
        complete_row_count: 2048,
        estimated_sequence_windows: 1024,
        symbols: vec![
            "005930".to_string(),
            "000660".to_string(),
            "AAPL".to_string(),
            "MSFT".to_string(),
        ],
        horizons: vec![4, 8, 16],
        window_lengths: vec![32, 64],
        outcome_label_distribution: BTreeMap::from([
            ("Win".to_string(), 420usize),
            ("Loss".to_string(), 360usize),
            ("NoTrade".to_string(), 244usize),
        ]),
        feature_schema_locked: true,
        no_lookahead_safe: true,
        storage_estimate_bytes: 524288,
        readiness_status: SequenceDatasetReadinessStatus::ReadyForSequenceDatasetExport,
        blockers: vec![],
        warnings: vec![],
        reason_codes: vec![],
    }
}

fn mamba_report(ready: bool) -> Mamba3ReadinessAuditV2 {
    Mamba3ReadinessAuditV2 {
        audit_id: "mamba".to_string(),
        dimension_results: vec![],
        sequence_readiness_report: sequence_report(),
        mamba3_runtime_present: false,
        rust_native_training_present: false,
        external_bridge_present: true,
        readiness_state: if ready {
            Mamba3ReadinessState::ReadyForExternalPrototype
        } else {
            Mamba3ReadinessState::BlockedByEvidenceDepth
        },
        final_recommendation: if ready {
            Mamba3ReadinessRecommendation::BuildExternalMamba3FinLitePrototype
        } else {
            Mamba3ReadinessRecommendation::ImproveEvidenceDepthFirst
        },
        blockers: vec![],
        warnings: vec![],
        reason_codes: vec![],
    }
}

fn decision(ready: bool) -> ModelEscalationDecisionV2 {
    ModelEscalationDecisionV2 {
        decision_id: "decision".to_string(),
        selected_candidate: if ready {
            ModelEscalationCandidate::ExternalMamba3FinLite
        } else {
            ModelEscalationCandidate::NoEscalation
        },
        rejected_candidates: vec![ModelEscalationCandidate::RustNativeMamba3Runtime],
        rationale: "prototype gate".to_string(),
        prerequisites: vec![],
        next_actions: vec![],
        decision_status: if ready {
            ModelEscalationDecisionStatus::ExternalMambaPrototypeAllowed
        } else {
            ModelEscalationDecisionStatus::NeedMoreEvidence
        },
        reason_codes: vec![],
    }
}

fn config(name: &str, ready: bool) -> Mamba3FinLitePrototypePlanConfig {
    let output_dir = common::sprint55_output_dir(name);
    let mamba_path = output_dir.join("mamba.json");
    let decision_path = output_dir.join("decision.json");
    fs::write(
        &mamba_path,
        mamba_report(ready).to_json_string().expect("mamba json"),
    )
    .expect("write mamba");
    fs::write(
        &decision_path,
        decision(ready).to_json_string().expect("decision json"),
    )
    .expect("write decision");
    Mamba3FinLitePrototypePlanConfig {
        plan_id: name.to_string(),
        mamba_readiness_report_paths: vec![mamba_path.display().to_string()],
        model_escalation_decision_paths: vec![decision_path.display().to_string()],
        output_root: output_dir.display().to_string(),
        reason_codes: vec![soma_zero::ReasonCode::DeterministicPath],
    }
}

#[test]
fn external_prototype_plan_can_be_generated_when_gate_passes() {
    let plan = Mamba3FinLitePrototypePlanRunner::default()
        .run(&config("prototype-plan-allowed", true))
        .expect("plan");
    assert!(plan.allowed);
    assert_eq!(
        plan.backend,
        Mamba3FinLitePrototypeBackend::ExternalPythonResearch
    );
    assert!(
        plan.required_prediction_schema
            .contains(&"p_win".to_string())
    );
    assert!(
        plan.model_card_requirements
            .iter()
            .any(|item| item.contains("feature schema"))
    );
}

#[test]
fn prototype_plan_forbids_live_trading_and_rust_runtime_work() {
    let plan = Mamba3FinLitePrototypePlanRunner::default()
        .run(&config("prototype-plan-deferred", false))
        .expect("plan");
    assert!(!plan.allowed);
    assert_eq!(plan.backend, Mamba3FinLitePrototypeBackend::Deferred);
    assert!(
        plan.forbidden_actions
            .iter()
            .any(|item| item.contains("no live trading"))
    );
    assert!(
        plan.forbidden_actions
            .iter()
            .any(|item| item.contains("no Rust inference"))
    );
    assert!(
        plan.forbidden_actions
            .iter()
            .any(|item| item.contains("no Rust training"))
    );
}

#[test]
fn prototype_plan_is_deterministic() {
    let cfg = config("prototype-plan-deterministic", true);
    let left = Mamba3FinLitePrototypePlanRunner::default()
        .run(&cfg)
        .expect("left")
        .to_json_string()
        .expect("left json");
    let right = Mamba3FinLitePrototypePlanRunner::default()
        .run(&cfg)
        .expect("right")
        .to_json_string()
        .expect("right json");
    assert_eq!(left, right);
}

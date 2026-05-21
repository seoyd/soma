mod common;

use std::collections::BTreeMap;
use std::fs;

use serde_json::json;
use soma_zero::{
    CoreCompletionAuditReport, CoreCompletionRecommendation, CoreCompletionStatus,
    Mamba3ReadinessAuditV2, Mamba3ReadinessRecommendation, Mamba3ReadinessState,
    ModelEscalationCandidate, ModelEscalationDecisionRunner, ModelEscalationDecisionStatus,
    ModelEscalationDecisionV2Config, SequenceDatasetReadinessReport,
    SequenceDatasetReadinessStatus,
};

fn core_report(recommendation: CoreCompletionRecommendation) -> CoreCompletionAuditReport {
    CoreCompletionAuditReport {
        audit_id: "core".to_string(),
        maturity_matrix: soma_zero::CoreSubsystemMaturityMatrix {
            rows: vec![],
            research_ready_count: 0,
            paper_ready_count: 0,
            blocked_count: 0,
            deferred_count: 0,
            forbidden_count: 0,
            reason_codes: vec![],
        },
        passed_core_requirements: vec![],
        failed_core_requirements: vec![],
        missing_subsystems: vec![],
        blocked_subsystems: vec![],
        deferred_subsystems: vec![],
        forbidden_subsystems: vec![],
        core_completion_status: CoreCompletionStatus::CoreResearchOperatingSystemComplete,
        final_recommendation: recommendation,
        warnings: vec![],
        reason_codes: vec![],
    }
}

fn sequence_report(status: SequenceDatasetReadinessStatus) -> SequenceDatasetReadinessReport {
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
        readiness_status: status,
        blockers: vec![],
        warnings: vec![],
        reason_codes: vec![],
    }
}

fn mamba_report(
    state: Mamba3ReadinessState,
    recommendation: Mamba3ReadinessRecommendation,
) -> Mamba3ReadinessAuditV2 {
    Mamba3ReadinessAuditV2 {
        audit_id: "mamba".to_string(),
        dimension_results: vec![],
        sequence_readiness_report: sequence_report(
            SequenceDatasetReadinessStatus::ReadyForSequenceDatasetExport,
        ),
        mamba3_runtime_present: false,
        rust_native_training_present: false,
        external_bridge_present: true,
        readiness_state: state,
        final_recommendation: recommendation,
        blockers: vec![],
        warnings: vec![],
        reason_codes: vec![],
    }
}

fn config(
    name: &str,
    core: &CoreCompletionAuditReport,
    sequence: &SequenceDatasetReadinessReport,
    mamba: &Mamba3ReadinessAuditV2,
    support: serde_json::Value,
    prefer_external: bool,
) -> ModelEscalationDecisionV2Config {
    let output_dir = common::sprint55_output_dir(name);
    let core_path = output_dir.join("core.json");
    let sequence_path = output_dir.join("sequence.json");
    let mamba_path = output_dir.join("mamba.json");
    let support_path = output_dir.join("support.json");
    fs::write(&core_path, core.to_json_string().expect("core json")).expect("write core");
    fs::write(
        &sequence_path,
        sequence.to_json_string().expect("sequence json"),
    )
    .expect("write sequence");
    fs::write(&mamba_path, mamba.to_json_string().expect("mamba json")).expect("write mamba");
    fs::write(
        &support_path,
        serde_json::to_string_pretty(&support).expect("support json"),
    )
    .expect("write support");
    ModelEscalationDecisionV2Config {
        decision_id: name.to_string(),
        core_completion_audit_report_paths: vec![core_path.display().to_string()],
        sequence_readiness_report_paths: vec![sequence_path.display().to_string()],
        mamba_readiness_report_paths: vec![mamba_path.display().to_string()],
        supporting_artifact_paths: vec![support_path.display().to_string()],
        output_root: output_dir.display().to_string(),
        prefer_external_prototype: prefer_external,
        reason_codes: vec![soma_zero::ReasonCode::DeterministicPath],
    }
}

#[test]
fn insufficient_evidence_selects_need_more_evidence() {
    let decision = ModelEscalationDecisionRunner::default()
        .run(&config(
            "model-escalation-need-evidence",
            &core_report(CoreCompletionRecommendation::CoreNeedsOutcomeLinkDepth),
            &sequence_report(SequenceDatasetReadinessStatus::ReadyForSequenceDatasetExport),
            &mamba_report(
                Mamba3ReadinessState::BlockedByEvidenceDepth,
                Mamba3ReadinessRecommendation::ImproveEvidenceDepthFirst,
            ),
            json!({"kis_outcome_depth_bottleneck": true}),
            false,
        ))
        .expect("decision");
    assert_eq!(
        decision.decision_status,
        ModelEscalationDecisionStatus::NeedMoreEvidence
    );
}

#[test]
fn sequence_not_ready_selects_build_sequence_dataset_first() {
    let decision = ModelEscalationDecisionRunner::default()
        .run(&config(
            "model-escalation-sequence-first",
            &core_report(CoreCompletionRecommendation::KeepTrinity),
            &sequence_report(SequenceDatasetReadinessStatus::NeedMoreRows),
            &mamba_report(
                Mamba3ReadinessState::BlockedBySequenceDataset,
                Mamba3ReadinessRecommendation::BuildSequenceDatasetFirst,
            ),
            json!({}),
            false,
        ))
        .expect("decision");
    assert_eq!(
        decision.decision_status,
        ModelEscalationDecisionStatus::BuildSequenceDatasetFirst
    );
    assert_eq!(
        decision.selected_candidate,
        ModelEscalationCandidate::SequenceDatasetBuild
    );
}

#[test]
fn sequence_ready_can_select_external_mamba3fin_lite() {
    let decision = ModelEscalationDecisionRunner::default()
        .run(&config(
            "model-escalation-external-mamba",
            &core_report(CoreCompletionRecommendation::KeepTrinity),
            &sequence_report(SequenceDatasetReadinessStatus::ReadyForSequenceDatasetExport),
            &mamba_report(
                Mamba3ReadinessState::ReadyForExternalPrototype,
                Mamba3ReadinessRecommendation::BuildExternalMamba3FinLitePrototype,
            ),
            json!({"keep_committee_only": false}),
            true,
        ))
        .expect("decision");
    assert_eq!(
        decision.decision_status,
        ModelEscalationDecisionStatus::ExternalMambaPrototypeAllowed
    );
    assert_eq!(
        decision.selected_candidate,
        ModelEscalationCandidate::ExternalMamba3FinLite
    );
}

#[test]
fn rust_native_mamba_runtime_is_rejected() {
    let decision = ModelEscalationDecisionRunner::default()
        .run(&config(
            "model-escalation-reject-runtime",
            &core_report(CoreCompletionRecommendation::KeepTrinity),
            &sequence_report(SequenceDatasetReadinessStatus::ReadyForSequenceDatasetExport),
            &mamba_report(
                Mamba3ReadinessState::ReadyForExternalPrototype,
                Mamba3ReadinessRecommendation::BuildExternalMamba3FinLitePrototype,
            ),
            json!({}),
            true,
        ))
        .expect("decision");
    assert!(
        decision
            .rejected_candidates
            .contains(&ModelEscalationCandidate::RustNativeMamba3Runtime)
    );
}

#[test]
fn current_kis_outcome_depth_bottleneck_keeps_mamba_deferred() {
    let decision = ModelEscalationDecisionRunner::default()
        .run(&config(
            "model-escalation-kis-bottleneck",
            &core_report(CoreCompletionRecommendation::CoreNeedsKISEvidenceDepth),
            &sequence_report(SequenceDatasetReadinessStatus::ReadyForSequenceDatasetExport),
            &mamba_report(
                Mamba3ReadinessState::BlockedByEvidenceDepth,
                Mamba3ReadinessRecommendation::ImproveEvidenceDepthFirst,
            ),
            json!({"kis_outcome_depth_bottleneck": true}),
            false,
        ))
        .expect("decision");
    assert_eq!(
        decision.decision_status,
        ModelEscalationDecisionStatus::NeedMoreEvidence
    );
    assert_ne!(
        decision.selected_candidate,
        ModelEscalationCandidate::ExternalMamba3FinLite
    );
}

#[test]
fn model_escalation_decision_is_deterministic() {
    let cfg = config(
        "model-escalation-deterministic",
        &core_report(CoreCompletionRecommendation::KeepTrinity),
        &sequence_report(SequenceDatasetReadinessStatus::NeedMoreRows),
        &mamba_report(
            Mamba3ReadinessState::BlockedBySequenceDataset,
            Mamba3ReadinessRecommendation::BuildSequenceDatasetFirst,
        ),
        json!({}),
        false,
    );
    let left = ModelEscalationDecisionRunner::default()
        .run(&cfg)
        .expect("left")
        .to_json_string()
        .expect("left json");
    let right = ModelEscalationDecisionRunner::default()
        .run(&cfg)
        .expect("right")
        .to_json_string()
        .expect("right json");
    assert_eq!(left, right);
}

mod common;

use std::collections::BTreeMap;
use std::fs;

use serde_json::json;
use soma_zero::{
    CoreCompletionAuditReport, CoreCompletionRecommendation, CoreCompletionStatus,
    Mamba3ReadinessRecommendation, Mamba3ReadinessState, MambaReadinessV2Config,
    MambaReadinessV2Runner, SequenceDatasetReadinessReport, SequenceDatasetReadinessStatus,
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

fn config(
    name: &str,
    core: &CoreCompletionAuditReport,
    sequence: &SequenceDatasetReadinessReport,
    support: serde_json::Value,
    allow_external: bool,
) -> MambaReadinessV2Config {
    let output_dir = common::sprint55_output_dir(name);
    let core_path = output_dir.join("core.json");
    let sequence_path = output_dir.join("sequence.json");
    let support_path = output_dir.join("support.json");
    fs::write(&core_path, core.to_json_string().expect("core json")).expect("write core");
    fs::write(
        &sequence_path,
        sequence.to_json_string().expect("sequence json"),
    )
    .expect("write sequence");
    fs::write(
        &support_path,
        serde_json::to_string_pretty(&support).expect("support json"),
    )
    .expect("write support");
    MambaReadinessV2Config {
        audit_id: name.to_string(),
        sequence_readiness_report_paths: vec![sequence_path.display().to_string()],
        core_completion_audit_report_paths: vec![core_path.display().to_string()],
        supporting_artifact_paths: vec![support_path.display().to_string()],
        output_root: output_dir.display().to_string(),
        allow_external_prototype_only: allow_external,
        require_control_tower_visibility: true,
        require_risk_governor_integration: true,
        reason_codes: vec![soma_zero::ReasonCode::DeterministicPath],
    }
}

fn healthy_support() -> serde_json::Value {
    json!({
        "evidence_depth_sufficient": true,
        "counterfactual_depth_sufficient": true,
        "calibration_baseline_ready": true,
        "external_bridge_present": true,
        "inference_budget_ok": true,
        "risk_governor_integrated": true,
        "control_tower_visible": true,
        "mamba3_runtime_present": false,
        "rust_native_training_present": false,
        "signal_model_weak": false
    })
}

#[test]
fn runtime_and_training_absence_are_detected() {
    let report = MambaReadinessV2Runner::default()
        .run(&config(
            "mamba-absence",
            &core_report(CoreCompletionRecommendation::KeepTrinity),
            &sequence_report(SequenceDatasetReadinessStatus::ReadyForSequenceDatasetExport),
            healthy_support(),
            false,
        ))
        .expect("report");
    assert!(!report.mamba3_runtime_present);
    assert!(!report.rust_native_training_present);
}

#[test]
fn insufficient_evidence_blocks_mamba() {
    let report = MambaReadinessV2Runner::default()
        .run(&config(
            "mamba-evidence-blocked",
            &core_report(CoreCompletionRecommendation::CoreNeedsOutcomeLinkDepth),
            &sequence_report(SequenceDatasetReadinessStatus::ReadyForSequenceDatasetExport),
            json!({"evidence_depth_sufficient": false, "counterfactual_depth_sufficient": false}),
            false,
        ))
        .expect("report");
    assert_eq!(
        report.readiness_state,
        Mamba3ReadinessState::BlockedByEvidenceDepth
    );
    assert_eq!(
        report.final_recommendation,
        Mamba3ReadinessRecommendation::ImproveEvidenceDepthFirst
    );
}

#[test]
fn insufficient_sequence_dataset_blocks_mamba() {
    let report = MambaReadinessV2Runner::default()
        .run(&config(
            "mamba-sequence-blocked",
            &core_report(CoreCompletionRecommendation::KeepTrinity),
            &sequence_report(SequenceDatasetReadinessStatus::NeedMoreRows),
            healthy_support(),
            false,
        ))
        .expect("report");
    assert_eq!(
        report.readiness_state,
        Mamba3ReadinessState::BlockedBySequenceDataset
    );
}

#[test]
fn no_lookahead_failure_blocks_mamba() {
    let mut sequence = sequence_report(SequenceDatasetReadinessStatus::NeedNoLookaheadProof);
    sequence.no_lookahead_safe = false;
    let report = MambaReadinessV2Runner::default()
        .run(&config(
            "mamba-no-lookahead-blocked",
            &core_report(CoreCompletionRecommendation::KeepTrinity),
            &sequence,
            healthy_support(),
            false,
        ))
        .expect("report");
    assert_eq!(
        report.readiness_state,
        Mamba3ReadinessState::BlockedByNoLookahead
    );
}

#[test]
fn storage_failure_blocks_mamba() {
    let mut sequence = sequence_report(SequenceDatasetReadinessStatus::NeedStorageBudget);
    sequence.storage_estimate_bytes = 4_194_304;
    let report = MambaReadinessV2Runner::default()
        .run(&config(
            "mamba-storage-blocked",
            &core_report(CoreCompletionRecommendation::KeepTrinity),
            &sequence,
            healthy_support(),
            false,
        ))
        .expect("report");
    assert_eq!(
        report.readiness_state,
        Mamba3ReadinessState::BlockedByStorage
    );
}

#[test]
fn sequence_ready_fixture_allows_external_prototype_only() {
    let report = MambaReadinessV2Runner::default()
        .run(&config(
            "mamba-ready-external",
            &core_report(CoreCompletionRecommendation::KeepTrinity),
            &sequence_report(SequenceDatasetReadinessStatus::ReadyForSequenceDatasetExport),
            healthy_support(),
            true,
        ))
        .expect("report");
    assert_eq!(
        report.readiness_state,
        Mamba3ReadinessState::ReadyForExternalPrototype
    );
    assert_eq!(
        report.final_recommendation,
        Mamba3ReadinessRecommendation::BuildExternalMamba3FinLitePrototype
    );
}

#[test]
fn full_rust_runtime_remains_deferred_or_forbidden() {
    let report = MambaReadinessV2Runner::default()
        .run(&config(
            "mamba-runtime-forbidden",
            &core_report(CoreCompletionRecommendation::KeepTrinity),
            &sequence_report(SequenceDatasetReadinessStatus::ReadyForSequenceDatasetExport),
            json!({"mamba3_runtime_present": true, "rust_native_training_present": false}),
            false,
        ))
        .expect("report");
    assert!(matches!(
        report.readiness_state,
        Mamba3ReadinessState::Forbidden | Mamba3ReadinessState::RustRuntimeDeferred
    ));
}

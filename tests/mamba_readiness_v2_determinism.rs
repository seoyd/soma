mod common;

use std::collections::BTreeMap;
use std::fs;

use serde_json::json;
use soma_zero::{
    CoreCompletionAuditReport, CoreCompletionStatus, MambaReadinessV2Config,
    MambaReadinessV2Runner, SequenceDatasetReadinessReport, SequenceDatasetReadinessStatus,
};

#[test]
fn mamba_readiness_v2_is_deterministic() {
    let output_dir = common::sprint55_output_dir("mamba-v2-deterministic");
    let core = CoreCompletionAuditReport {
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
        final_recommendation: soma_zero::CoreCompletionRecommendation::KeepTrinity,
        warnings: vec![],
        reason_codes: vec![],
    };
    let sequence = SequenceDatasetReadinessReport {
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
    };
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
        serde_json::to_string_pretty(&json!({
            "evidence_depth_sufficient": true,
            "counterfactual_depth_sufficient": true,
            "calibration_baseline_ready": true,
            "external_bridge_present": true,
            "inference_budget_ok": true,
            "risk_governor_integrated": true,
            "control_tower_visible": true,
            "mamba3_runtime_present": false,
            "rust_native_training_present": false
        }))
        .expect("support json"),
    )
    .expect("write support");
    let config = MambaReadinessV2Config {
        audit_id: "deterministic".to_string(),
        sequence_readiness_report_paths: vec![sequence_path.display().to_string()],
        core_completion_audit_report_paths: vec![core_path.display().to_string()],
        supporting_artifact_paths: vec![support_path.display().to_string()],
        output_root: output_dir.display().to_string(),
        allow_external_prototype_only: true,
        require_control_tower_visibility: true,
        require_risk_governor_integration: true,
        reason_codes: vec![soma_zero::ReasonCode::DeterministicPath],
    };
    let left = MambaReadinessV2Runner::default()
        .run(&config)
        .expect("left")
        .to_json_string()
        .expect("left json");
    let right = MambaReadinessV2Runner::default()
        .run(&config)
        .expect("right")
        .to_json_string()
        .expect("right json");
    assert_eq!(left, right);
}

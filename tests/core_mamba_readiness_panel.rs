use std::collections::BTreeMap;

use soma_zero::{
    CoreCompletionAuditReport, CoreCompletionRecommendation, CoreCompletionStatus,
    Mamba3ReadinessAuditV2, Mamba3ReadinessRecommendation, Mamba3ReadinessState,
    ModelEscalationCandidate, ModelEscalationDecisionStatus, ModelEscalationDecisionV2,
    SequenceDatasetReadinessReport, SequenceDatasetReadinessStatus,
    build_core_mamba_readiness_panel,
};

#[test]
fn panel_displays_core_mamba_sequence_and_model_status_without_runtime_claims() {
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
        failed_core_requirements: vec!["NeedMoreOutcomeLinkDepth".to_string()],
        missing_subsystems: vec![],
        blocked_subsystems: vec![],
        deferred_subsystems: vec![],
        forbidden_subsystems: vec![],
        core_completion_status: CoreCompletionStatus::CoreResearchOperatingSystemComplete,
        final_recommendation: CoreCompletionRecommendation::CoreNeedsOutcomeLinkDepth,
        warnings: vec![
            "core completion does not imply live trading readiness or profitability".to_string(),
        ],
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
        readiness_status: SequenceDatasetReadinessStatus::NeedMoreRows,
        blockers: vec!["NeedMoreRows".to_string()],
        warnings: vec![],
        reason_codes: vec![],
    };
    let mamba = Mamba3ReadinessAuditV2 {
        audit_id: "mamba".to_string(),
        dimension_results: vec![],
        sequence_readiness_report: sequence.clone(),
        mamba3_runtime_present: false,
        rust_native_training_present: false,
        external_bridge_present: true,
        readiness_state: Mamba3ReadinessState::BlockedByEvidenceDepth,
        final_recommendation: Mamba3ReadinessRecommendation::ImproveEvidenceDepthFirst,
        blockers: vec!["NeedMoreKISEvidence".to_string()],
        warnings: vec![],
        reason_codes: vec![],
    };
    let model = ModelEscalationDecisionV2 {
        decision_id: "decision".to_string(),
        selected_candidate: ModelEscalationCandidate::NoEscalation,
        rejected_candidates: vec![ModelEscalationCandidate::RustNativeMamba3Runtime],
        rationale: "hold".to_string(),
        prerequisites: vec!["stronger KIS evidence depth".to_string()],
        next_actions: vec!["improve outcome-link depth first".to_string()],
        decision_status: ModelEscalationDecisionStatus::NeedMoreEvidence,
        reason_codes: vec![],
    };

    let panel =
        build_core_mamba_readiness_panel(Some(&core), Some(&mamba), Some(&sequence), Some(&model))
            .expect("panel");
    let text = panel.to_text();

    assert!(text.contains("core_completion_status=CoreResearchOperatingSystemComplete"));
    assert!(text.contains("mamba3_readiness_state=BlockedByEvidenceDepth"));
    assert!(text.contains("sequence_dataset_status=NeedMoreRows"));
    assert!(text.contains("selected_model_escalation_decision=NoEscalation"));
    assert!(text.contains("mamba_runtime_implemented=false"));
    assert!(text.contains("train_button=false"));
    assert!(text.contains("live_button=false"));
}

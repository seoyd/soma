use soma_zero::{
    AiSignalStatus, InferenceMode, Mamba3FinCandidateReadiness, Mamba3FinCandidateSpec,
    Mamba3RequirementStatus, OfficialConsistencyStatus, SequenceDatasetConfig, SequenceDatasetSpec,
    build_mamba3fin_candidate_report, build_mamba3fin_gap_analysis,
};

fn valid_sequence_spec() -> SequenceDatasetSpec {
    SequenceDatasetSpec {
        config: SequenceDatasetConfig {
            window_size: 32,
            stride: 1,
            horizon_bars: 8,
            max_windows: 256,
            max_bytes: 1_048_576,
            ..SequenceDatasetConfig::default()
        },
        estimated_windows: 64,
        estimated_bytes: 8_192,
        no_lookahead_safe: true,
        storage_budget_ok: true,
        reason_codes: vec![],
    }
}

#[test]
fn gap_analysis_marks_missing_mamba_math_but_keeps_bridge_ready() {
    let report = build_mamba3fin_gap_analysis(
        None,
        Some(OfficialConsistencyStatus::ConsistentEnough),
        Some(&valid_sequence_spec()),
    );

    let missing_math = report
        .requirements
        .iter()
        .find(|item| item.requirement_id == "Mamba3SpecificMath")
        .expect("mamba math requirement");
    let external_bridge = report
        .requirements
        .iter()
        .find(|item| item.requirement_id == "ExternalPredictionBridge")
        .expect("external bridge requirement");
    let trading_heads = report
        .requirements
        .iter()
        .find(|item| item.requirement_id == "TradingHeads")
        .expect("trading heads requirement");
    let risk_governor = report
        .requirements
        .iter()
        .find(|item| item.requirement_id == "RiskGovernorIntegration")
        .expect("risk governor requirement");

    assert_eq!(missing_math.status, Mamba3RequirementStatus::Missing);
    assert_eq!(external_bridge.status, Mamba3RequirementStatus::Satisfied);
    assert_eq!(trading_heads.status, Mamba3RequirementStatus::Satisfied);
    assert_eq!(risk_governor.status, Mamba3RequirementStatus::Satisfied);
    assert!(
        report
            .blockers
            .iter()
            .any(|item| item.contains("complex-valued"))
    );
}

#[test]
fn candidate_spec_defaults_to_external_prototype_only() {
    let spec = Mamba3FinCandidateSpec::default_external();

    assert_eq!(spec.inference_mode, InferenceMode::ExternalPredictionFile);
    assert!(spec.use_complex_state);
    assert!(spec.use_mimo);
    assert!(!spec.use_micro_attention);
}

#[test]
fn candidate_report_stays_conservative_without_strong_evidence() {
    let spec = valid_sequence_spec();
    let gap = build_mamba3fin_gap_analysis(
        None,
        Some(OfficialConsistencyStatus::ConsistentEnough),
        Some(&spec),
    );

    let report = build_mamba3fin_candidate_report(
        &gap,
        OfficialConsistencyStatus::ConsistentEnough,
        Some(AiSignalStatus::PipelineOnly),
        Some(&spec),
    );

    assert_eq!(report.readiness, Mamba3FinCandidateReadiness::DoNotBuildYet);
}

#[test]
fn candidate_report_requires_sequence_dataset_first_when_missing() {
    let gap = build_mamba3fin_gap_analysis(
        None,
        Some(OfficialConsistencyStatus::ConsistentEnough),
        None,
    );

    let report = build_mamba3fin_candidate_report(
        &gap,
        OfficialConsistencyStatus::ConsistentEnough,
        Some(AiSignalStatus::ExternalModelEvaluated),
        None,
    );

    assert_eq!(
        report.readiness,
        Mamba3FinCandidateReadiness::BuildSequenceDatasetFirst
    );
}

#[test]
fn gap_analysis_text_is_deterministic() {
    let spec = valid_sequence_spec();
    let left = build_mamba3fin_gap_analysis(
        None,
        Some(OfficialConsistencyStatus::ConsistentEnough),
        Some(&spec),
    );
    let right = build_mamba3fin_gap_analysis(
        None,
        Some(OfficialConsistencyStatus::ConsistentEnough),
        Some(&spec),
    );

    assert_eq!(left.to_text(), right.to_text());
}

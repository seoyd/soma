use soma_zero::{
    AiSignalStatus, InferenceMode, Mamba3FinCandidateReadiness, Mamba3FinCandidateReport,
    Mamba3FinCandidateSpec, ModelEscalationDecision, ModelEscalationGate,
    ModelEscalationGateConfig, ModelEscalationGateResult, OfficialConsistencyStatus,
    SequenceDatasetConfig, SequenceDatasetSpec,
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

fn candidate_report() -> Mamba3FinCandidateReport {
    Mamba3FinCandidateReport {
        candidate_spec: Mamba3FinCandidateSpec::default_external(),
        readiness: Mamba3FinCandidateReadiness::BuildExternalPrototype,
        blockers: vec![],
        reason_codes: vec![],
    }
}

#[test]
fn missing_official_data_requires_improving_data_first() {
    let result = ModelEscalationGateResult::evaluate(
        &ModelEscalationGateConfig::default(),
        OfficialConsistencyStatus::MissingAuth,
        Some(AiSignalStatus::ExternalModelEvaluated),
        40,
        0.05,
        true,
        true,
        Some(&valid_sequence_spec()),
        &candidate_report(),
    );

    assert_eq!(
        result.decision,
        ModelEscalationDecision::ImproveOfficialDataFirst
    );
}

#[test]
fn poor_calibration_requires_improving_signal_model_first() {
    let result = ModelEscalationGateResult::evaluate(
        &ModelEscalationGateConfig::default(),
        OfficialConsistencyStatus::ConsistentEnough,
        Some(AiSignalStatus::PoorCalibration),
        40,
        0.20,
        true,
        true,
        Some(&valid_sequence_spec()),
        &candidate_report(),
    );

    assert_eq!(
        result.decision,
        ModelEscalationDecision::ImproveSignalModelFirst
    );
}

#[test]
fn poor_risk_behavior_requires_improving_risk_governor_first() {
    let result = ModelEscalationGateResult::evaluate(
        &ModelEscalationGateConfig::default(),
        OfficialConsistencyStatus::ConsistentEnough,
        Some(AiSignalStatus::RejectedByRisk),
        40,
        0.05,
        false,
        true,
        Some(&valid_sequence_spec()),
        &candidate_report(),
    );

    assert_eq!(
        result.decision,
        ModelEscalationDecision::ImproveRiskGovernorFirst
    );
}

#[test]
fn missing_sequence_spec_requires_building_dataset_first() {
    let result = ModelEscalationGateResult::evaluate(
        &ModelEscalationGateConfig::default(),
        OfficialConsistencyStatus::ConsistentEnough,
        Some(AiSignalStatus::ExternalModelEvaluated),
        40,
        0.05,
        true,
        true,
        None,
        &candidate_report(),
    );

    assert_eq!(
        result.decision,
        ModelEscalationDecision::BuildSequenceDatasetFirst
    );
    assert!(
        result
            .failed_gates
            .contains(&ModelEscalationGate::SequenceSpec)
    );
}

#[test]
fn all_gates_can_pass_for_external_prototype_only() {
    let result = ModelEscalationGateResult::evaluate(
        &ModelEscalationGateConfig::default(),
        OfficialConsistencyStatus::ConsistentEnough,
        Some(AiSignalStatus::ExternalModelEvaluated),
        40,
        0.05,
        true,
        true,
        Some(&valid_sequence_spec()),
        &candidate_report(),
    );

    assert_eq!(
        result.decision,
        ModelEscalationDecision::BuildMamba3FinExternalPrototype
    );
    assert!(
        result
            .passed_gates
            .contains(&ModelEscalationGate::RustNativeDeferred)
    );
}

#[test]
fn crypto_only_prototype_requires_explicit_flag() {
    let blocked = ModelEscalationGateResult::evaluate(
        &ModelEscalationGateConfig::default(),
        OfficialConsistencyStatus::CryptoOnly,
        Some(AiSignalStatus::ExternalModelEvaluated),
        40,
        0.05,
        true,
        true,
        Some(&valid_sequence_spec()),
        &candidate_report(),
    );
    assert_eq!(blocked.decision, ModelEscalationDecision::Blocked);
    assert!(
        blocked
            .failed_gates
            .contains(&ModelEscalationGate::CryptoOnlyPrototypeAllowed)
    );

    let allowed = ModelEscalationGateResult::evaluate(
        &ModelEscalationGateConfig {
            allow_mamba3_prototype_without_equity_data: true,
            ..ModelEscalationGateConfig::default()
        },
        OfficialConsistencyStatus::CryptoOnly,
        Some(AiSignalStatus::ExternalModelEvaluated),
        40,
        0.05,
        true,
        true,
        Some(&valid_sequence_spec()),
        &candidate_report(),
    );
    assert_eq!(
        allowed.decision,
        ModelEscalationDecision::BuildMamba3FinExternalPrototype
    );
    assert!(
        allowed
            .warnings
            .iter()
            .any(|value| value.contains("crypto-only"))
    );
}

#[test]
fn default_candidate_never_selects_rust_native_inference() {
    let candidate = candidate_report();

    assert_eq!(
        candidate.candidate_spec.inference_mode,
        InferenceMode::ExternalPredictionFile
    );
}

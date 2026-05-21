use soma_zero::{
    AiSignalDecisionInputs, AiSignalStatus, AiSignalUsefulnessReport, CalibrationSummary,
    ModelComparisonSummary, ModelUsefulnessGateConfig, ModelUsefulnessGateInputs,
    ModelUsefulnessGateResult, OfficialDatasetCoverageReport, PerformanceSummary,
    RiskGovernorSummary, StorageBudgetSummary,
};

fn gate_result(passed: bool) -> ModelUsefulnessGateResult {
    let inputs = ModelUsefulnessGateInputs {
        schema_valid: passed,
        outcome_count: if passed { 50 } else { 0 },
        calibration_count: if passed { 50 } else { 0 },
        brier_score: if passed { 0.1 } else { 0.5 },
        expected_calibration_error: if passed { 0.05 } else { 0.5 },
        selected_profit_factor: Some(1.2),
        delta_max_drawdown_pct: Some(0.0),
        delta_net_return_pct: Some(0.02),
        denial_rate: 0.2,
        approval_rate: 0.3,
        emergency_stop_count: 0,
        leakage_detected: false,
        data_quality_score: 0.95,
        budget_exceeded: false,
    };
    ModelUsefulnessGateResult::evaluate(&ModelUsefulnessGateConfig::default(), &inputs)
}

fn coverage() -> OfficialDatasetCoverageReport {
    OfficialDatasetCoverageReport {
        total_ready_entries: 1,
        crypto_ready_entries: 1,
        korean_equity_ready_entries: 0,
        us_equity_ready_entries: 0,
        skipped_missing_auth_entries: 0,
        skipped_budget_entries: 0,
        failed_preflight_entries: 0,
        provider_statuses: Default::default(),
        missing_auth_providers: Vec::new(),
        compactness_summary: "compact-only".to_string(),
        non_official_ready_entries: 0,
        reason_codes: vec![],
    }
}

fn inputs() -> AiSignalDecisionInputs {
    AiSignalDecisionInputs {
        official_dataset_count: 1,
        total_outcome_records: 50,
        baseline_summary: PerformanceSummary {
            dataset_count: 1,
            total_trades: 50,
            avg_net_return_pct: 0.01,
            avg_profit_factor: 1.1,
            avg_max_drawdown_pct: 0.05,
        },
        external_summary: None,
        calibration_summary: CalibrationSummary {
            total_count: 50,
            avg_brier_score: 0.1,
            avg_expected_calibration_error: 0.05,
            acceptable: true,
        },
        risk_governor_summary: RiskGovernorSummary {
            total_signals: 100,
            denied_by_risk: 20,
            denial_rate: 0.2,
            approval_rate: 0.3,
            emergency_stop_count: 0,
            cooldown_count: 0,
            defensive_value: 1.0,
            opportunity_cost: 0.2,
            stable: true,
        },
        model_comparison_summary: None,
        storage_budget_summary: StorageBudgetSummary {
            collection_bytes: 100,
            dataset_export_bytes: 20,
            prediction_bytes: 0,
            report_bytes: 10,
            budget_exceeded: false,
        },
        has_external_evaluation: false,
        comparison_external_better: false,
        missing_auth: false,
        non_official_ready_entries: 0,
        allow_upbit_only: true,
        allow_equity_missing_auth: true,
        min_official_ready_datasets: 1,
    }
}

#[test]
fn no_official_data_yields_missing_official_data() {
    let mut decision_inputs = inputs();
    decision_inputs.official_dataset_count = 0;
    let report = AiSignalUsefulnessReport::decide(&coverage(), &gate_result(true), decision_inputs);
    assert_eq!(report.status, AiSignalStatus::MissingOfficialData);
}

#[test]
fn only_mock_data_yields_pipeline_only() {
    let mut decision_inputs = inputs();
    decision_inputs.official_dataset_count = 0;
    decision_inputs.non_official_ready_entries = 1;
    let report = AiSignalUsefulnessReport::decide(&coverage(), &gate_result(true), decision_inputs);
    assert_eq!(report.status, AiSignalStatus::PipelineOnly);
}

#[test]
fn baseline_only_with_enough_outcomes_is_baseline_evaluated() {
    let report = AiSignalUsefulnessReport::decide(&coverage(), &gate_result(true), inputs());
    assert_eq!(report.status, AiSignalStatus::BaselineEvaluated);
}

#[test]
fn poor_calibration_blocks_usefulness() {
    let gate = soma_zero::ModelUsefulnessGateResult::evaluate(
        &ModelUsefulnessGateConfig::default(),
        &ModelUsefulnessGateInputs {
            schema_valid: true,
            outcome_count: 50,
            calibration_count: 50,
            brier_score: 0.5,
            expected_calibration_error: 0.5,
            selected_profit_factor: Some(1.2),
            delta_max_drawdown_pct: Some(0.0),
            delta_net_return_pct: Some(0.02),
            denial_rate: 0.2,
            approval_rate: 0.3,
            emergency_stop_count: 0,
            leakage_detected: false,
            data_quality_score: 0.95,
            budget_exceeded: false,
        },
    );
    let report = AiSignalUsefulnessReport::decide(&coverage(), &gate, inputs());
    assert_eq!(report.status, AiSignalStatus::PoorCalibration);
}

#[test]
fn worse_than_baseline_is_reported() {
    let mut decision_inputs = inputs();
    decision_inputs.has_external_evaluation = true;
    decision_inputs.external_summary = Some(decision_inputs.baseline_summary.clone());
    decision_inputs.model_comparison_summary = Some(ModelComparisonSummary {
        compared_datasets: 1,
        external_better_count: 0,
        avg_delta_net_return_pct: -0.01,
        avg_delta_max_drawdown_pct: 0.0,
        avg_delta_profit_factor: -0.1,
    });
    let gate = soma_zero::ModelUsefulnessGateResult::evaluate(
        &ModelUsefulnessGateConfig::default(),
        &ModelUsefulnessGateInputs {
            delta_net_return_pct: Some(-0.01),
            schema_valid: true,
            outcome_count: 50,
            calibration_count: 50,
            brier_score: 0.1,
            expected_calibration_error: 0.03,
            selected_profit_factor: Some(1.0),
            delta_max_drawdown_pct: Some(0.0),
            denial_rate: 0.2,
            approval_rate: 0.2,
            emergency_stop_count: 0,
            leakage_detected: false,
            data_quality_score: 0.95,
            budget_exceeded: false,
        },
    );
    let report = AiSignalUsefulnessReport::decide(&coverage(), &gate, decision_inputs);
    assert_eq!(report.status, AiSignalStatus::WorseThanBaseline);
}

#[test]
fn good_external_evidence_can_be_useful_candidate() {
    let mut decision_inputs = inputs();
    decision_inputs.has_external_evaluation = true;
    decision_inputs.external_summary = Some(PerformanceSummary {
        dataset_count: 1,
        total_trades: 50,
        avg_net_return_pct: 0.03,
        avg_profit_factor: 1.5,
        avg_max_drawdown_pct: 0.04,
    });
    decision_inputs.model_comparison_summary = Some(ModelComparisonSummary {
        compared_datasets: 1,
        external_better_count: 1,
        avg_delta_net_return_pct: 0.02,
        avg_delta_max_drawdown_pct: -0.01,
        avg_delta_profit_factor: 0.3,
    });
    decision_inputs.comparison_external_better = true;
    let report = AiSignalUsefulnessReport::decide(&coverage(), &gate_result(true), decision_inputs);
    assert_eq!(report.status, AiSignalStatus::UsefulCandidate);
}

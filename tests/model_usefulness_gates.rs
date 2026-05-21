use soma_zero::{ModelUsefulnessGate, ModelUsefulnessGateConfig, ModelUsefulnessGateInputs};

fn good_inputs() -> ModelUsefulnessGateInputs {
    ModelUsefulnessGateInputs {
        schema_valid: true,
        outcome_count: 50,
        calibration_count: 50,
        brier_score: 0.10,
        expected_calibration_error: 0.03,
        selected_profit_factor: Some(1.5),
        delta_max_drawdown_pct: Some(0.0),
        delta_net_return_pct: Some(0.02),
        denial_rate: 0.20,
        approval_rate: 0.30,
        emergency_stop_count: 0,
        leakage_detected: false,
        data_quality_score: 0.95,
        budget_exceeded: false,
    }
}

#[test]
fn schema_mismatch_fails_gate() {
    let mut inputs = good_inputs();
    inputs.schema_valid = false;
    let result = soma_zero::ModelUsefulnessGateResult::evaluate(
        &ModelUsefulnessGateConfig::default(),
        &inputs,
    );
    assert!(!result.passed);
    assert!(
        result
            .failed_gates
            .contains(&ModelUsefulnessGate::SchemaValid)
    );
}

#[test]
fn too_few_outcomes_fail_gate() {
    let mut inputs = good_inputs();
    inputs.outcome_count = 3;
    let result = soma_zero::ModelUsefulnessGateResult::evaluate(
        &ModelUsefulnessGateConfig::default(),
        &inputs,
    );
    assert!(!result.passed);
    assert!(
        result
            .failed_gates
            .contains(&ModelUsefulnessGate::EnoughOutcomes)
    );
}

#[test]
fn worse_drawdown_fails_gate() {
    let mut inputs = good_inputs();
    inputs.delta_max_drawdown_pct = Some(0.10);
    let result = soma_zero::ModelUsefulnessGateResult::evaluate(
        &ModelUsefulnessGateConfig::default(),
        &inputs,
    );
    assert!(!result.passed);
    assert!(
        result
            .failed_gates
            .contains(&ModelUsefulnessGate::DrawdownNotWorse)
    );
}

#[test]
fn all_good_inputs_pass_gate() {
    let result = soma_zero::ModelUsefulnessGateResult::evaluate(
        &ModelUsefulnessGateConfig::default(),
        &good_inputs(),
    );
    assert!(result.passed);
    assert!(result.failed_gates.is_empty());
}

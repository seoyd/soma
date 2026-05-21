use soma_zero::{
    AblationDelta, AblationDimension, AblationInterpretationFlag, AblationResultStatus,
    AblationVariantResult, NextStepRecommendation, build_sensitivity_summary, select_next_step,
};

fn result(
    variant_id: &str,
    dimension: AblationDimension,
    target: &str,
    flags: Vec<AblationInterpretationFlag>,
    delta: AblationDelta,
) -> AblationVariantResult {
    AblationVariantResult {
        variant_id: variant_id.to_string(),
        matrix_id: format!("matrix-{variant_id}"),
        dimension,
        status: AblationResultStatus::Applied,
        overrides: vec![soma_zero::AblationOverride {
            target: target.to_string(),
            value: soma_zero::AblationValue::Float(1.0),
        }],
        flags,
        reason_codes: vec![],
        delta,
        avg_calibration_brier: None,
        report: None,
    }
}

#[test]
fn next_step_prefers_data_quality_first_when_quality_sensitivity_dominates() {
    let results = vec![result(
        "quality",
        AblationDimension::RiskGovernor,
        "min_data_quality",
        vec![],
        AblationDelta {
            avg_net_return_pct: 0.02,
            ..AblationDelta::default()
        },
    )];
    let summary = build_sensitivity_summary(&results);
    assert_eq!(
        select_next_step(&results, &summary),
        NextStepRecommendation::ImproveDataFirst
    );
}

#[test]
fn next_step_tightens_risk_gates_on_cost_fragility() {
    let results = vec![result(
        "fragile-cost",
        AblationDimension::CostModel,
        "spread_bps",
        vec![AblationInterpretationFlag::HighFragility],
        AblationDelta {
            avg_net_return_pct: -0.01,
            avg_max_drawdown_pct: 0.01,
            ..AblationDelta::default()
        },
    )];
    let summary = build_sensitivity_summary(&results);
    assert_eq!(
        select_next_step(&results, &summary),
        NextStepRecommendation::TightenRiskGates
    );
}

#[test]
fn next_step_refines_feature_set_on_stable_feature_gain() {
    let results = vec![result(
        "feature-gain",
        AblationDimension::FeatureGroup,
        "volume",
        vec![AblationInterpretationFlag::CandidateImprovement],
        AblationDelta {
            avg_net_return_pct: 0.01,
            ..AblationDelta::default()
        },
    )];
    let summary = build_sensitivity_summary(&results);
    assert_eq!(
        select_next_step(&results, &summary),
        NextStepRecommendation::RefineFeatureSet
    );
}

#[test]
fn next_step_revisits_labels_on_robust_triple_barrier_change() {
    let results = vec![result(
        "label-shift",
        AblationDimension::TripleBarrier,
        "take_profit_pct",
        vec![],
        AblationDelta {
            avg_net_return_pct: 0.0,
            ..AblationDelta::default()
        },
    )];
    let summary = build_sensitivity_summary(&results);
    assert_eq!(
        select_next_step(&results, &summary),
        NextStepRecommendation::RevisitLabels
    );
}

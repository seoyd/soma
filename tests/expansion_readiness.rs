use soma_zero::experiment::readiness::{
    ExpansionReadinessDecision, PersonaReadinessSummary, build_expansion_readiness_report,
};
use soma_zero::{
    DataQualityAggregate, DataQualitySeverity, ExperimentRunKey, ExperimentRunStatus,
    ExperimentRunSummary, ModelComparisonAggregate, ReasonCode, RiskGovernorAggregate,
};

fn run_summary(
    dataset_id: &str,
    status: ExperimentRunStatus,
    total_decisions: usize,
    denied_trades: usize,
    net_return_pct: f64,
) -> ExperimentRunSummary {
    ExperimentRunSummary {
        run_key: ExperimentRunKey {
            dataset_id: dataset_id.to_string(),
            variant_id: "baseline".to_string(),
            experiment_id: format!("{dataset_id}-baseline"),
        },
        status,
        manifest_summary: String::new(),
        data_quality_score: 0.95,
        data_quality_severity: DataQualitySeverity::Good,
        total_decisions,
        executed_trades: total_decisions.saturating_sub(denied_trades),
        denied_trades,
        no_trades: 0,
        net_return_pct,
        max_drawdown_pct: 0.05,
        profit_factor: Some(1.2),
        calibration_brier: Some(0.10),
        risk_defensive_value: Some(0.0),
        external_better: None,
        reason_codes: vec![ReasonCode::DeterministicPath],
    }
}

fn data_quality(
    dataset_count: usize,
    bad_count: usize,
    unusable_count: usize,
) -> DataQualityAggregate {
    DataQualityAggregate {
        dataset_count,
        good_count: dataset_count.saturating_sub(bad_count + unusable_count),
        warning_count: 0,
        bad_count,
        unusable_count,
        avg_data_quality_score: 0.95,
        worst_dataset_id: None,
        common_reason_codes: vec![],
        gap_heavy_datasets: vec![],
        duplicate_heavy_datasets: vec![],
        invalid_ohlc_datasets: vec![],
        reason_codes: vec![ReasonCode::DeterministicPath],
    }
}

fn risk(total_emergency_stops: usize) -> RiskGovernorAggregate {
    RiskGovernorAggregate {
        total_denials: 0,
        total_cooldowns: 0,
        total_emergency_stops,
        avoided_loss_count: 0,
        missed_gain_count: 0,
        defensive_value_total: 0.0,
        opportunity_cost_total: 0.0,
        most_common_denial_reasons: vec![],
        deny_rate_by_dataset: std::collections::BTreeMap::new(),
        reason_codes: vec![ReasonCode::DeterministicPath],
    }
}

fn model(compared_runs: usize, external_validated: bool) -> ModelComparisonAggregate {
    ModelComparisonAggregate {
        compared_runs,
        external_better_count: 0,
        baseline_better_count: 0,
        tie_count: compared_runs,
        avg_delta_net_return_pct: 0.0,
        avg_delta_max_drawdown_pct: 0.0,
        avg_delta_calibration_brier: 0.0,
        external_failed_schema_count: if external_validated { 0 } else { 1 },
        external_missing_prediction_count: 0,
        reason_codes: vec![ReasonCode::DeterministicPath],
    }
}

fn persona(redundancy_warning: bool, expansion_recommended: bool) -> PersonaReadinessSummary {
    PersonaReadinessSummary {
        current_persona_count: 3,
        selected_vote_counts: std::collections::BTreeMap::new(),
        forced_contrarian_counts: std::collections::BTreeMap::new(),
        average_contribution_scores: std::collections::BTreeMap::new(),
        high_confidence_miss_counts: std::collections::BTreeMap::new(),
        persona_signal_correlation_proxy: Some(if redundancy_warning { 0.95 } else { 0.40 }),
        redundancy_warning,
        expansion_recommended,
        reason_codes: vec![ReasonCode::DeterministicPath],
    }
}

#[test]
fn expansion_readiness_decisions_stay_conservative() {
    let no_bundles = Vec::<&soma_zero::ExperimentReportBundle>::new();

    let insufficient = build_expansion_readiness_report(
        &[run_summary("a", ExperimentRunStatus::Passed, 5, 0, 0.02)],
        &no_bundles,
        &data_quality(1, 0, 0),
        &risk(0),
        &model(0, false),
        &persona(false, false),
    );
    assert_eq!(
        insufficient.decision,
        ExpansionReadinessDecision::NeedMoreExperiments
    );

    let bad_data = build_expansion_readiness_report(
        &[run_summary("a", ExperimentRunStatus::Failed, 30, 0, 0.02)],
        &no_bundles,
        &data_quality(3, 1, 0),
        &risk(0),
        &model(0, false),
        &persona(false, false),
    );
    assert_eq!(
        bad_data.decision,
        ExpansionReadinessDecision::ImproveDataFirst
    );

    let unstable_risk = build_expansion_readiness_report(
        &[
            run_summary("a", ExperimentRunStatus::Passed, 20, 20, 0.02),
            run_summary("b", ExperimentRunStatus::Passed, 20, 20, 0.01),
        ],
        &no_bundles,
        &data_quality(2, 0, 0),
        &risk(0),
        &model(0, false),
        &persona(false, false),
    );
    assert_eq!(
        unstable_risk.decision,
        ExpansionReadinessDecision::ImproveRiskGovernorFirst
    );

    let poor_signal = build_expansion_readiness_report(
        &[
            run_summary("a", ExperimentRunStatus::Passed, 20, 1, -0.20),
            run_summary("b", ExperimentRunStatus::Passed, 20, 1, -0.15),
        ],
        &no_bundles,
        &data_quality(2, 0, 0),
        &risk(0),
        &model(0, false),
        &persona(false, false),
    );
    assert_eq!(
        poor_signal.decision,
        ExpansionReadinessDecision::ImproveSignalModelFirst
    );

    let redundant = build_expansion_readiness_report(
        &[
            run_summary("a", ExperimentRunStatus::Passed, 20, 1, 0.03),
            run_summary("b", ExperimentRunStatus::Passed, 20, 1, 0.02),
        ],
        &no_bundles,
        &data_quality(2, 0, 0),
        &risk(0),
        &model(0, false),
        &persona(true, false),
    );
    assert_eq!(
        redundant.decision,
        ExpansionReadinessDecision::HoldCurrentScope
    );

    let expand = build_expansion_readiness_report(
        &[
            run_summary("a", ExperimentRunStatus::Passed, 20, 1, 0.03),
            run_summary("b", ExperimentRunStatus::Passed, 20, 1, 0.02),
            run_summary("c", ExperimentRunStatus::Passed, 20, 1, 0.04),
        ],
        &no_bundles,
        &data_quality(3, 0, 0),
        &risk(0),
        &model(2, true),
        &persona(false, true),
    );
    assert_eq!(
        expand.decision,
        ExpansionReadinessDecision::ExpandToSixPersonas
    );
}

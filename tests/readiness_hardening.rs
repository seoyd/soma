use soma_zero::ReasonCode;
use soma_zero::experiment::campaign::{
    CampaignAggregate, CampaignMatrixResult, CampaignMatrixStatus, ResearchCampaignConfig,
};
use soma_zero::experiment::diff::build_campaign_diff_report;
use soma_zero::experiment::readiness::{
    ExpansionReadinessDecision, build_campaign_expansion_readiness_report,
};
use soma_zero::experiment::regression::{
    RegressionGuardConfig, RegressionGuardResult, evaluate_regression_guard,
};

fn aggregate() -> CampaignAggregate {
    CampaignAggregate {
        campaign_id: "campaign".to_string(),
        matrix_count: 2,
        total_runs: 6,
        passed_runs: 4,
        failed_runs: 0,
        skipped_runs: 0,
        usable_dataset_count: 2,
        total_dataset_count: 2,
        total_outcome_records: 40,
        total_executed_trades: 20,
        total_no_trades: 4,
        total_denials: 4,
        average_data_quality_score: 0.90,
        worst_data_quality_score: 0.85,
        average_net_return_pct: 0.03,
        median_net_return_pct: 0.03,
        worst_net_return_pct: 0.00,
        average_max_drawdown_pct: 0.05,
        worst_max_drawdown_pct: 0.08,
        average_profit_factor: Some(1.2),
        average_calibration_brier: Some(0.10),
        regime_coverage_count: 2,
        unknown_regime_rate: 0.0,
        panic_regime_rate: 0.0,
        risk_defensive_value_total: 1.0,
        risk_opportunity_cost_total: 0.0,
        persona_redundancy_warning_count: 0,
        external_model_validated_count: 1,
        reason_codes: vec![ReasonCode::DeterministicPath],
    }
}

fn matrix_results() -> Vec<CampaignMatrixResult> {
    vec![CampaignMatrixResult {
        matrix_id: "m1".to_string(),
        source: "embedded:m1".to_string(),
        status: CampaignMatrixStatus::Passed,
        report: None,
        reason_codes: vec![ReasonCode::DeterministicPath],
        error: None,
    }]
}

#[test]
fn readiness_hardening_stays_conservative_and_blocks_regression() {
    let config = ResearchCampaignConfig::default();
    let mut current = aggregate();
    let previous = aggregate();
    current.usable_dataset_count = 1;
    let diff = build_campaign_diff_report(&current, Some(&previous), Some("prev"));
    let guard = RegressionGuardResult {
        passed: true,
        regressions: vec![],
        warnings: vec![],
        reason_codes: vec![ReasonCode::DeterministicPath],
    };
    let report = build_campaign_expansion_readiness_report(
        &config,
        &current,
        &matrix_results(),
        &diff,
        &guard,
    );
    assert_eq!(
        report.decision,
        ExpansionReadinessDecision::NeedMoreExperiments
    );

    current = aggregate();
    current.average_data_quality_score = 0.50;
    let diff = build_campaign_diff_report(&current, Some(&previous), Some("prev"));
    let report = build_campaign_expansion_readiness_report(
        &config,
        &current,
        &matrix_results(),
        &diff,
        &guard,
    );
    assert_eq!(
        report.decision,
        ExpansionReadinessDecision::ImproveDataFirst
    );

    current = aggregate();
    current.total_denials = 39;
    current.total_no_trades = 39;
    let diff = build_campaign_diff_report(&current, Some(&previous), Some("prev"));
    let report = build_campaign_expansion_readiness_report(
        &config,
        &current,
        &matrix_results(),
        &diff,
        &guard,
    );
    assert_eq!(
        report.decision,
        ExpansionReadinessDecision::ImproveRiskGovernorFirst
    );

    current = aggregate();
    current.average_net_return_pct = -0.20;
    current.worst_net_return_pct = -0.25;
    let diff = build_campaign_diff_report(&current, Some(&previous), Some("prev"));
    let report = build_campaign_expansion_readiness_report(
        &config,
        &current,
        &matrix_results(),
        &diff,
        &guard,
    );
    assert_eq!(
        report.decision,
        ExpansionReadinessDecision::ImproveSignalModelFirst
    );

    current = aggregate();
    current.persona_redundancy_warning_count = 2;
    let diff = build_campaign_diff_report(&current, Some(&previous), Some("prev"));
    let report = build_campaign_expansion_readiness_report(
        &config,
        &current,
        &matrix_results(),
        &diff,
        &guard,
    );
    assert_eq!(
        report.decision,
        ExpansionReadinessDecision::HoldCurrentScope
    );

    current = aggregate();
    let diff = build_campaign_diff_report(&current, Some(&previous), Some("prev"));
    let guard = RegressionGuardResult {
        passed: false,
        regressions: vec![],
        warnings: vec![],
        reason_codes: vec![ReasonCode::RegressionDetected],
    };
    let report = build_campaign_expansion_readiness_report(
        &config,
        &current,
        &matrix_results(),
        &diff,
        &guard,
    );
    assert_eq!(
        report.decision,
        ExpansionReadinessDecision::RegressedSinceLastCampaign
    );

    let expand_config = ResearchCampaignConfig {
        allow_persona_expansion_recommendation: true,
        min_usable_datasets: 2,
        min_total_outcome_records: 20,
        min_regime_coverage_count: 2,
        min_passed_runs: 2,
        ..ResearchCampaignConfig::default()
    };
    current = aggregate();
    let better_previous = CampaignAggregate {
        average_net_return_pct: 0.01,
        worst_max_drawdown_pct: 0.10,
        ..aggregate()
    };
    let diff = build_campaign_diff_report(&current, Some(&better_previous), Some("prev"));
    let guard = evaluate_regression_guard(
        &RegressionGuardConfig::default(),
        &current,
        Some(&better_previous),
        &diff,
    );
    let report = build_campaign_expansion_readiness_report(
        &expand_config,
        &current,
        &matrix_results(),
        &diff,
        &guard,
    );
    assert_eq!(
        report.decision,
        ExpansionReadinessDecision::ExpandToSixPersonas
    );
}

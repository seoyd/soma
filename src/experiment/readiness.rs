use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::core::ReasonCode;

use super::aggregate::{
    DataQualityAggregate, ExperimentRunSummary, ModelComparisonAggregate, RiskGovernorAggregate,
    persona_metrics, primary_report,
};
use super::report_bundle::ExperimentReportBundle;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PersonaReadinessSummary {
    pub current_persona_count: usize,
    pub selected_vote_counts: BTreeMap<String, usize>,
    pub forced_contrarian_counts: BTreeMap<String, usize>,
    pub average_contribution_scores: BTreeMap<String, f64>,
    pub high_confidence_miss_counts: BTreeMap<String, usize>,
    pub persona_signal_correlation_proxy: Option<f64>,
    pub redundancy_warning: bool,
    pub expansion_recommended: bool,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExpansionReadinessDecision {
    HoldCurrentScope,
    ExpandToSixPersonas,
    ImproveDataFirst,
    ImproveRiskGovernorFirst,
    ImproveSignalModelFirst,
    NeedMoreExperiments,
    Blocked,
    RegressedSinceLastCampaign,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ExpansionReadinessEvidence {
    pub dataset_count: usize,
    pub usable_dataset_count: usize,
    pub total_outcome_records: usize,
    pub stable_feature_schema: bool,
    pub leakage_guard_passed: bool,
    pub risk_governor_stable: bool,
    pub persona_redundancy_low: bool,
    pub baseline_not_catastrophic: bool,
    pub external_model_validated: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ExpansionReadinessReport {
    pub decision: ExpansionReadinessDecision,
    pub confidence: f64,
    pub evidence: ExpansionReadinessEvidence,
    pub blockers: Vec<String>,
    pub warnings: Vec<String>,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CampaignExpansionReadinessEvidence {
    pub matrix_count: usize,
    pub total_dataset_count: usize,
    pub usable_dataset_count: usize,
    pub total_outcome_records: usize,
    pub passed_runs: usize,
    pub regime_coverage_count: usize,
    pub average_data_quality_score: f64,
    pub average_calibration_brier: Option<f64>,
    pub worst_max_drawdown_pct: f64,
    pub risk_defensive_value_total: f64,
    pub denial_rate: f64,
    pub no_trade_rate: f64,
    pub persona_redundancy_warning_rate: f64,
    pub risk_governor_not_blocking_everything: bool,
    pub risk_governor_not_allowing_everything: bool,
    pub stable_feature_schema: bool,
    pub leakage_guard_passed: bool,
    pub no_runtime_llm: bool,
    pub no_real_broker: bool,
    pub no_live_api: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CampaignExpansionReadinessReport {
    pub decision: ExpansionReadinessDecision,
    pub confidence: f64,
    pub evidence: CampaignExpansionReadinessEvidence,
    pub blockers: Vec<String>,
    pub warnings: Vec<String>,
    pub reason_codes: Vec<ReasonCode>,
}

pub fn build_persona_readiness_summary(
    bundles: &[&ExperimentReportBundle],
) -> PersonaReadinessSummary {
    let mut selected_vote_counts = BTreeMap::new();
    let mut forced_contrarian_counts = BTreeMap::new();
    let mut contribution_sums = BTreeMap::<String, (f64, usize)>::new();
    let mut high_confidence_miss_counts = BTreeMap::new();
    for bundle in bundles {
        for persona in persona_metrics(bundle) {
            *selected_vote_counts
                .entry(persona.persona_id.clone())
                .or_insert(0) += persona.selected_count;
            *forced_contrarian_counts
                .entry(persona.persona_id.clone())
                .or_insert(0) += persona.forced_contrarian_count;
            let entry = contribution_sums
                .entry(persona.persona_id.clone())
                .or_insert((0.0, 0));
            entry.0 += persona.avg_contribution_score;
            entry.1 += 1;
            *high_confidence_miss_counts
                .entry(persona.persona_id.clone())
                .or_insert(0) += persona.high_confidence_miss_count;
        }
    }
    let average_contribution_scores = contribution_sums
        .into_iter()
        .map(|(persona, (sum, count))| (persona, if count == 0 { 0.0 } else { sum / count as f64 }))
        .collect::<BTreeMap<_, _>>();
    let current_persona_count = selected_vote_counts
        .keys()
        .chain(forced_contrarian_counts.keys())
        .collect::<BTreeSet<_>>()
        .len();
    let contribution_values = average_contribution_scores
        .values()
        .copied()
        .collect::<Vec<_>>();
    let correlation_proxy = if contribution_values.len() < 2 {
        Some(1.0)
    } else {
        let max = contribution_values
            .iter()
            .copied()
            .max_by(|left, right| left.total_cmp(right))
            .unwrap_or(0.0);
        let min = contribution_values
            .iter()
            .copied()
            .min_by(|left, right| left.total_cmp(right))
            .unwrap_or(0.0);
        Some(1.0 - (max - min).abs().min(1.0))
    };
    let total_selected = selected_vote_counts.values().sum::<usize>();
    let redundancy_warning = total_selected == 0
        || correlation_proxy.unwrap_or(1.0) >= 0.90
        || contribution_values.len() < 3;
    let enough_samples = bundles
        .iter()
        .filter_map(|bundle| primary_report(bundle))
        .map(|report| report.aggregate_metrics.trade_metrics.total_trades)
        .sum::<usize>()
        >= 20;
    let poor_quality = bundles.iter().any(|bundle| {
        matches!(
            bundle.data_quality_report.severity,
            crate::data::DataQualitySeverity::Bad | crate::data::DataQualitySeverity::Unusable
        )
    });
    let expansion_recommended = enough_samples && !poor_quality && !redundancy_warning;
    let mut reason_codes = vec![ReasonCode::DeterministicPath];
    if redundancy_warning {
        reason_codes.push(ReasonCode::ComparisonNotConclusive);
    }
    PersonaReadinessSummary {
        current_persona_count,
        selected_vote_counts,
        forced_contrarian_counts,
        average_contribution_scores,
        high_confidence_miss_counts,
        persona_signal_correlation_proxy: correlation_proxy,
        redundancy_warning,
        expansion_recommended,
        reason_codes,
    }
}

pub fn build_expansion_readiness_report(
    run_summaries: &[ExperimentRunSummary],
    bundles: &[&ExperimentReportBundle],
    data_quality: &DataQualityAggregate,
    risk: &RiskGovernorAggregate,
    model: &ModelComparisonAggregate,
    persona: &PersonaReadinessSummary,
) -> ExpansionReadinessReport {
    let feature_hashes = bundles
        .iter()
        .filter_map(|bundle| primary_report(bundle).map(|report| report.feature_schema.checksum))
        .collect::<BTreeSet<_>>();
    let stable_feature_schema = feature_hashes.len() <= 1;
    let leakage_guard_passed = bundles.iter().all(|bundle| {
        primary_report(bundle)
            .map(|report| {
                report
                    .folds
                    .iter()
                    .all(|fold| !fold.leakage_report.has_leakage)
            })
            .unwrap_or(true)
    });
    let usable_dataset_count = data_quality
        .dataset_count
        .saturating_sub(data_quality.bad_count + data_quality.unusable_count);
    let total_outcome_records = run_summaries
        .iter()
        .map(|summary| summary.total_decisions)
        .sum();
    let risk_governor_stable = risk.total_emergency_stops == 0
        && !run_summaries.is_empty()
        && average(
            &run_summaries
                .iter()
                .map(|summary| {
                    if summary.total_decisions == 0 {
                        1.0
                    } else {
                        summary.denied_trades as f64 / summary.total_decisions as f64
                    }
                })
                .collect::<Vec<_>>(),
        ) < 0.95;
    let baseline_not_catastrophic = !run_summaries.is_empty()
        && average(
            &run_summaries
                .iter()
                .map(|summary| summary.net_return_pct)
                .collect::<Vec<_>>(),
        ) > -0.10;
    let external_model_validated =
        model.compared_runs > 0 && model.external_failed_schema_count == 0;
    let evidence = ExpansionReadinessEvidence {
        dataset_count: data_quality.dataset_count,
        usable_dataset_count,
        total_outcome_records,
        stable_feature_schema,
        leakage_guard_passed,
        risk_governor_stable,
        persona_redundancy_low: !persona.redundancy_warning,
        baseline_not_catastrophic,
        external_model_validated,
    };
    let mut blockers = Vec::new();
    let mut warnings = Vec::new();
    let (decision, confidence, mut reason_codes) =
        if data_quality.unusable_count > 0 || data_quality.bad_count > 0 {
            blockers.push("data quality is not stable enough across datasets".to_string());
            (
                ExpansionReadinessDecision::ImproveDataFirst,
                0.85,
                vec![ReasonCode::DataUnusable],
            )
        } else if data_quality.dataset_count < 2 || total_outcome_records < 20 {
            warnings.push("not enough dataset coverage or outcome records".to_string());
            (
                ExpansionReadinessDecision::NeedMoreExperiments,
                0.80,
                vec![ReasonCode::ComparisonNotConclusive],
            )
        } else if !risk_governor_stable {
            blockers.push("risk governor behavior is not yet stable across runs".to_string());
            (
                ExpansionReadinessDecision::ImproveRiskGovernorFirst,
                0.80,
                vec![ReasonCode::DataQualityGateBreached],
            )
        } else if !baseline_not_catastrophic {
            blockers.push("signal performance is too weak or too unstable".to_string());
            (
                ExpansionReadinessDecision::ImproveSignalModelFirst,
                0.78,
                vec![ReasonCode::ExpectedEdgeBelowThreshold],
            )
        } else if persona.redundancy_warning {
            warnings.push("current persona structure still looks redundant".to_string());
            (
                ExpansionReadinessDecision::HoldCurrentScope,
                0.82,
                vec![ReasonCode::ComparisonNotConclusive],
            )
        } else if evidence.stable_feature_schema
            && evidence.leakage_guard_passed
            && evidence.risk_governor_stable
            && evidence.persona_redundancy_low
            && evidence.baseline_not_catastrophic
            && evidence.external_model_validated
            && usable_dataset_count >= 3
        {
            (
                ExpansionReadinessDecision::ExpandToSixPersonas,
                0.72,
                vec![ReasonCode::FeatureSchemaValidated],
            )
        } else {
            warnings.push("evidence is still mixed; stay conservative".to_string());
            (
                ExpansionReadinessDecision::NeedMoreExperiments,
                0.70,
                vec![ReasonCode::ComparisonNotConclusive],
            )
        };
    if !evidence.stable_feature_schema {
        warnings.push("feature schema differs across runs".to_string());
    }
    if !evidence.leakage_guard_passed {
        blockers.push("leakage guard did not remain clean across all runs".to_string());
        reason_codes.push(ReasonCode::LeakageDetected);
    }
    ExpansionReadinessReport {
        decision,
        confidence,
        evidence,
        blockers,
        warnings,
        reason_codes,
    }
}

fn average(values: &[f64]) -> f64 {
    if values.is_empty() {
        0.0
    } else {
        values.iter().sum::<f64>() / values.len() as f64
    }
}

pub fn build_campaign_expansion_readiness_report(
    config: &super::campaign::ResearchCampaignConfig,
    aggregate: &super::campaign::CampaignAggregate,
    matrix_results: &[super::campaign::CampaignMatrixResult],
    diff: &super::diff::CampaignDiffReport,
    regression_guard: &super::regression::RegressionGuardResult,
) -> CampaignExpansionReadinessReport {
    let stable_feature_schema = matrix_results.iter().all(|result| {
        result
            .report
            .as_ref()
            .map(|report| report.expansion_readiness.evidence.stable_feature_schema)
            .unwrap_or(true)
    });
    let leakage_guard_passed = matrix_results.iter().all(|result| {
        result
            .report
            .as_ref()
            .map(|report| report.expansion_readiness.evidence.leakage_guard_passed)
            .unwrap_or(true)
    });
    let denial_rate = safe_ratio(
        aggregate.total_denials as f64,
        aggregate.total_outcome_records as f64,
    );
    let no_trade_rate = safe_ratio(
        aggregate.total_no_trades as f64,
        aggregate.total_outcome_records as f64,
    );
    let persona_redundancy_warning_rate = safe_ratio(
        aggregate.persona_redundancy_warning_count as f64,
        aggregate.matrix_count as f64,
    );
    let risk_governor_not_blocking_everything = denial_rate < 0.95 && no_trade_rate < 0.995;
    let risk_governor_not_allowing_everything = aggregate.total_executed_trades == 0
        || aggregate.risk_defensive_value_total > 0.0
        || denial_rate > 0.0;
    let evidence = CampaignExpansionReadinessEvidence {
        matrix_count: aggregate.matrix_count,
        total_dataset_count: aggregate.total_dataset_count,
        usable_dataset_count: aggregate.usable_dataset_count,
        total_outcome_records: aggregate.total_outcome_records,
        passed_runs: aggregate.passed_runs,
        regime_coverage_count: aggregate.regime_coverage_count,
        average_data_quality_score: aggregate.average_data_quality_score,
        average_calibration_brier: aggregate.average_calibration_brier,
        worst_max_drawdown_pct: aggregate.worst_max_drawdown_pct,
        risk_defensive_value_total: aggregate.risk_defensive_value_total,
        denial_rate,
        no_trade_rate,
        persona_redundancy_warning_rate,
        risk_governor_not_blocking_everything,
        risk_governor_not_allowing_everything,
        stable_feature_schema,
        leakage_guard_passed,
        no_runtime_llm: true,
        no_real_broker: true,
        no_live_api: true,
    };
    let mut blockers = Vec::new();
    let mut warnings = Vec::new();
    let (decision, confidence, mut reason_codes) = if diff.comparable && !regression_guard.passed {
        blockers.push("campaign regressed versus previous evidence baseline".to_string());
        (
            ExpansionReadinessDecision::RegressedSinceLastCampaign,
            0.90,
            vec![ReasonCode::RegressionDetected],
        )
    } else if evidence.usable_dataset_count < config.min_usable_datasets
        || evidence.passed_runs < config.min_passed_runs
        || evidence.regime_coverage_count < config.min_regime_coverage_count
    {
        warnings.push(
            "campaign does not yet cover enough usable datasets, passed runs, or regimes"
                .to_string(),
        );
        (
            ExpansionReadinessDecision::NeedMoreExperiments,
            0.86,
            vec![ReasonCode::ComparisonNotConclusive],
        )
    } else if evidence.total_outcome_records < config.min_total_outcome_records {
        warnings.push("campaign outcome count is still too small".to_string());
        (
            ExpansionReadinessDecision::NeedMoreExperiments,
            0.84,
            vec![ReasonCode::ComparisonNotConclusive],
        )
    } else if evidence.average_data_quality_score < config.min_data_quality_score {
        blockers.push("campaign data quality is below the hard gate".to_string());
        (
            ExpansionReadinessDecision::ImproveDataFirst,
            0.88,
            vec![ReasonCode::DataQualityGateBreached],
        )
    } else if !evidence.risk_governor_not_blocking_everything
        || !evidence.risk_governor_not_allowing_everything
    {
        blockers
            .push("risk governor behavior is not stable enough across campaign runs".to_string());
        (
            ExpansionReadinessDecision::ImproveRiskGovernorFirst,
            0.87,
            vec![ReasonCode::DataQualityGateBreached],
        )
    } else if aggregate.average_net_return_pct <= -0.10 || aggregate.worst_net_return_pct <= -0.20 {
        blockers.push("campaign signal metrics are too weak or too negative".to_string());
        (
            ExpansionReadinessDecision::ImproveSignalModelFirst,
            0.85,
            vec![ReasonCode::ExpectedEdgeBelowThreshold],
        )
    } else if evidence.persona_redundancy_warning_rate > 0.50 {
        warnings.push("persona contribution still looks too redundant".to_string());
        (
            ExpansionReadinessDecision::HoldCurrentScope,
            0.83,
            vec![ReasonCode::ComparisonNotConclusive],
        )
    } else if !evidence.stable_feature_schema || !evidence.leakage_guard_passed {
        blockers.push("feature schema or leakage guarantees are not stable enough".to_string());
        (
            ExpansionReadinessDecision::Blocked,
            0.92,
            vec![ReasonCode::LeakageDetected],
        )
    } else if !config.allow_persona_expansion_recommendation {
        warnings
            .push("campaign config keeps persona expansion recommendation disabled".to_string());
        (
            ExpansionReadinessDecision::NeedMoreExperiments,
            0.80,
            vec![ReasonCode::ComparisonNotConclusive],
        )
    } else {
        (
            ExpansionReadinessDecision::ExpandToSixPersonas,
            0.70,
            vec![ReasonCode::FeatureSchemaValidated],
        )
    };
    if evidence
        .average_calibration_brier
        .is_some_and(|value| value > 0.30)
    {
        warnings.push("average calibration brier is still high".to_string());
    }
    if evidence.worst_max_drawdown_pct > config.max_allowed_drawdown_regression_pct.max(0.10) {
        warnings.push("worst drawdown remains elevated".to_string());
    }
    if diff
        .reason_codes
        .contains(&ReasonCode::CampaignDiffUnavailable)
    {
        reason_codes.push(ReasonCode::CampaignDiffUnavailable);
    }
    CampaignExpansionReadinessReport {
        decision,
        confidence,
        evidence,
        blockers,
        warnings,
        reason_codes,
    }
}

fn safe_ratio(numerator: f64, denominator: f64) -> f64 {
    if denominator <= 0.0 {
        0.0
    } else {
        numerator / denominator
    }
}

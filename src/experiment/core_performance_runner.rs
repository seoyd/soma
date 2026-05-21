use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::core::{
    ArtifactSize, CoreCheckConfig, CoreCheckRunner, CoreLatencyBudgetConfig,
    CoreLatencyBudgetReport, build_core_latency_budget_report,
};
use crate::experiment::{
    CoreBottleneckInputs, CorePerformanceArtifactInventory, CorePerformanceRegressionConfig,
    CorePerformanceRegressionReport, CorePerformanceRegressionSummary, CorePerformanceScorecard,
    CorePerformanceScorecardBundle, CorePerformanceScorecardConfig, SignalQualityInputs,
    build_core_bottleneck_report, build_core_performance_regression_report,
    build_core_performance_scorecard, build_signal_quality_report, summary_from_scorecard,
};
use crate::league::{
    CommitteeBenchmarkBundle, CommitteeBenchmarkConfig, CommitteeBenchmarkRunner,
    CommitteeCounterfactualAuditReport, CommitteeCounterfactualType,
    CommitteeOfficialBenchmarkConfig, CommitteeOfficialBenchmarkReport,
    CommitteeOfficialBenchmarkRunner, CommitteeOutcomeCoverageBundle,
    CommitteeOutcomeCoverageConfig, CommitteeOutcomeCoverageRunner, CommitteeReferencePackBundle,
    CommitteeReferencePackConfig, CommitteeReferencePackRunner, CommitteeValueAttributionInputs,
    CommitteeValueAttributionStatus, OfficialEvidenceReplicationBundle,
    OfficialEvidenceReplicationConfig, OfficialEvidenceReplicationReport,
    OfficialEvidenceReplicationRunner, build_committee_value_attribution_report,
};
use crate::risk::{
    NoTradeValueInputs, RiskGovernorValueInputs, build_no_trade_value_report,
    build_risk_governor_value_report,
};
use crate::{
    CommitteeAttributionStatus, CommitteeFinalAction, CoreLatencyBudgetStatus, CoreReadinessReport,
    RiskDecisionKind, SourceAwareBenchmarkConfig, SourceAwareBenchmarkReport,
    SourceAwareBenchmarkRunner, YahooResearchEvidenceConfig, YahooResearchEvidenceReport,
    YahooResearchEvidenceRunner,
};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CorePerformanceScorecardRunner;

#[derive(Default)]
struct LoadedCorePerformanceArtifacts {
    inventory_entries: Vec<(
        String,
        Option<crate::experiment::CorePerformanceArtifactKind>,
        Option<Value>,
    )>,
    core_checks: Vec<CoreReadinessReport>,
    official_replications: Vec<OfficialEvidenceReplicationReport>,
    official_benchmarks: Vec<CommitteeOfficialBenchmarkReport>,
    outcome_coverage_bundles: Vec<CommitteeOutcomeCoverageBundle>,
    reference_pack_bundles: Vec<CommitteeReferencePackBundle>,
    benchmark_bundles: Vec<CommitteeBenchmarkBundle>,
    source_benchmarks: Vec<SourceAwareBenchmarkReport>,
    yahoo_reports: Vec<YahooResearchEvidenceReport>,
    load_warnings: Vec<String>,
}

impl CorePerformanceScorecardRunner {
    pub fn run(
        &self,
        config: &CorePerformanceScorecardConfig,
    ) -> Result<CorePerformanceScorecardBundle, String> {
        config.validate()?;
        let loaded = self.load_artifacts(config);
        let artifact_inventory =
            CorePerformanceArtifactInventory::from_resolved_entries(&loaded.inventory_entries);
        let load_warnings = loaded.load_warnings.clone();
        let signal_quality_report =
            build_signal_quality_report(&build_signal_inputs(config, &artifact_inventory, &loaded));
        let committee_value_attribution_report = build_committee_value_attribution_report(
            &build_committee_value_inputs(&artifact_inventory, &loaded),
        );
        let risk_governor_value_report = build_risk_governor_value_report(&build_risk_inputs(
            &artifact_inventory,
            &loaded,
            &committee_value_attribution_report,
        ));
        let no_trade_value_report =
            build_no_trade_value_report(&build_no_trade_inputs(&artifact_inventory, &loaded));
        let latency_budget_report = build_latency_budget_report(
            config,
            &artifact_inventory,
            &signal_quality_report,
            &committee_value_attribution_report,
            &risk_governor_value_report,
            &no_trade_value_report,
        );

        let bottleneck_inputs = build_bottleneck_inputs(
            &artifact_inventory,
            &loaded,
            &signal_quality_report,
            &committee_value_attribution_report,
            &risk_governor_value_report,
            &no_trade_value_report,
            &latency_budget_report,
        );
        let bottleneck_report = build_core_bottleneck_report(&bottleneck_inputs);

        let mut scorecard = build_core_performance_scorecard(
            config,
            artifact_inventory,
            signal_quality_report,
            committee_value_attribution_report,
            risk_governor_value_report,
            no_trade_value_report,
            latency_budget_report,
            None,
            bottleneck_report,
            load_warnings.clone(),
        );

        let regression_report = build_regression_report(config, &scorecard)?;
        scorecard.regression_report = regression_report;
        let updated_scorecard = build_core_performance_scorecard(
            config,
            scorecard.artifact_inventory.clone(),
            scorecard.signal_quality_report.clone(),
            scorecard.committee_value_attribution_report.clone(),
            scorecard.risk_governor_value_report.clone(),
            scorecard.no_trade_value_report.clone(),
            scorecard.latency_budget_report.clone(),
            scorecard.regression_report.clone(),
            scorecard.bottleneck_report.clone(),
            load_warnings,
        );

        let output_dir = config.output_dir();
        let written_files = BTreeMap::from([
            (
                "artifact_inventory".to_string(),
                output_dir
                    .join("artifact_inventory.txt")
                    .display()
                    .to_string(),
            ),
            (
                "signal_quality".to_string(),
                output_dir.join("signal_quality.txt").display().to_string(),
            ),
            (
                "committee_value_attribution".to_string(),
                output_dir
                    .join("committee_value_attribution.txt")
                    .display()
                    .to_string(),
            ),
            (
                "risk_governor_value".to_string(),
                output_dir
                    .join("risk_governor_value.txt")
                    .display()
                    .to_string(),
            ),
            (
                "no_trade_value".to_string(),
                output_dir.join("no_trade_value.txt").display().to_string(),
            ),
            (
                "latency_budget".to_string(),
                output_dir.join("latency_budget.txt").display().to_string(),
            ),
            (
                "regression_guard".to_string(),
                output_dir
                    .join("regression_guard.txt")
                    .display()
                    .to_string(),
            ),
            (
                "bottleneck_report".to_string(),
                output_dir
                    .join("bottleneck_report.txt")
                    .display()
                    .to_string(),
            ),
            (
                "core_performance_scorecard".to_string(),
                output_dir
                    .join("core_performance_scorecard.txt")
                    .display()
                    .to_string(),
            ),
        ]);
        let bundle = CorePerformanceScorecardBundle {
            scorecard: updated_scorecard,
            output_dir: output_dir.display().to_string(),
            written_files,
            reason_codes: vec![crate::core::ReasonCode::CorePerformanceBundleBuilt],
        };
        bundle.write_to_dir(&output_dir)?;
        Ok(bundle)
    }

    fn load_artifacts(
        &self,
        config: &CorePerformanceScorecardConfig,
    ) -> LoadedCorePerformanceArtifacts {
        let mut loaded = LoadedCorePerformanceArtifacts::default();
        for path in &config.core_check_report_paths {
            match load_core_check(path) {
                Ok(Some(report)) => {
                    loaded.inventory_entries.push((
                        path.clone(),
                        Some(crate::experiment::CorePerformanceArtifactKind::CoreCheckReport),
                        serde_json::to_value(&report).ok(),
                    ));
                    loaded.core_checks.push(report);
                }
                Ok(None) => {
                    loaded.inventory_entries.push((
                        path.clone(),
                        Some(crate::experiment::CorePerformanceArtifactKind::CoreCheckReport),
                        None,
                    ));
                    loaded
                        .load_warnings
                        .push(format!("skipped core-check artifact: {path}"));
                }
                Err(err) => {
                    loaded.inventory_entries.push((
                        path.clone(),
                        Some(crate::experiment::CorePerformanceArtifactKind::CoreCheckReport),
                        None,
                    ));
                    loaded
                        .load_warnings
                        .push(format!("failed to load core-check artifact {path}: {err}"));
                }
            }
        }
        for path in &config.official_replication_report_paths {
            match load_official_replication(path) {
                Ok(Some(report)) => {
                    loaded.inventory_entries.push((
                        path.clone(),
                        Some(crate::experiment::CorePerformanceArtifactKind::OfficialReplicationReport),
                        serde_json::to_value(&report).ok(),
                    ));
                    loaded.official_replications.push(report);
                }
                Ok(None) => {
                    loaded.inventory_entries.push((
                        path.clone(),
                        Some(crate::experiment::CorePerformanceArtifactKind::OfficialReplicationReport),
                        None,
                    ));
                    loaded
                        .load_warnings
                        .push(format!("skipped official replication artifact: {path}"));
                }
                Err(err) => {
                    loaded.inventory_entries.push((
                        path.clone(),
                        Some(crate::experiment::CorePerformanceArtifactKind::OfficialReplicationReport),
                        None,
                    ));
                    loaded.load_warnings.push(format!(
                        "failed to load official replication artifact {path}: {err}"
                    ));
                }
            }
        }
        for path in &config.committee_official_benchmark_paths {
            match load_official_benchmark(path) {
                Ok(Some(report)) => {
                    loaded.inventory_entries.push((
                        path.clone(),
                        Some(
                            crate::experiment::CorePerformanceArtifactKind::OfficialCommitteeBenchmarkReport,
                        ),
                        serde_json::to_value(&report).ok(),
                    ));
                    loaded.official_benchmarks.push(report);
                }
                Ok(None) => {
                    loaded.inventory_entries.push((
                        path.clone(),
                        Some(
                            crate::experiment::CorePerformanceArtifactKind::OfficialCommitteeBenchmarkReport,
                        ),
                        None,
                    ));
                    loaded.load_warnings.push(format!(
                        "skipped committee official benchmark artifact: {path}"
                    ));
                }
                Err(err) => {
                    loaded.inventory_entries.push((
                        path.clone(),
                        Some(
                            crate::experiment::CorePerformanceArtifactKind::OfficialCommitteeBenchmarkReport,
                        ),
                        None,
                    ));
                    loaded.load_warnings.push(format!(
                        "failed to load committee official benchmark artifact {path}: {err}"
                    ));
                }
            }
        }
        for path in &config.committee_outcome_coverage_paths {
            match load_outcome_coverage(path) {
                Ok(Some(bundle)) => {
                    loaded.inventory_entries.push((
                        path.clone(),
                        Some(crate::experiment::CorePerformanceArtifactKind::CommitteeOutcomeCoverageBundle),
                        serde_json::to_value(&bundle).ok(),
                    ));
                    loaded.outcome_coverage_bundles.push(bundle);
                }
                Ok(None) => {
                    loaded.inventory_entries.push((
                        path.clone(),
                        Some(crate::experiment::CorePerformanceArtifactKind::CommitteeOutcomeCoverageBundle),
                        None,
                    ));
                    loaded.load_warnings.push(format!(
                        "skipped committee outcome coverage artifact: {path}"
                    ));
                }
                Err(err) => {
                    loaded.inventory_entries.push((
                        path.clone(),
                        Some(crate::experiment::CorePerformanceArtifactKind::CommitteeOutcomeCoverageBundle),
                        None,
                    ));
                    loaded.load_warnings.push(format!(
                        "failed to load committee outcome coverage artifact {path}: {err}"
                    ));
                }
            }
        }
        for path in &config.committee_reference_pack_paths {
            match load_reference_pack(path) {
                Ok(Some(bundle)) => {
                    loaded.inventory_entries.push((
                        path.clone(),
                        Some(crate::experiment::CorePerformanceArtifactKind::CommitteeReferencePackBundle),
                        serde_json::to_value(&bundle).ok(),
                    ));
                    loaded.reference_pack_bundles.push(bundle);
                }
                Ok(None) => {
                    loaded.inventory_entries.push((
                        path.clone(),
                        Some(crate::experiment::CorePerformanceArtifactKind::CommitteeReferencePackBundle),
                        None,
                    ));
                    loaded
                        .load_warnings
                        .push(format!("skipped committee reference pack artifact: {path}"));
                }
                Err(err) => {
                    loaded.inventory_entries.push((
                        path.clone(),
                        Some(crate::experiment::CorePerformanceArtifactKind::CommitteeReferencePackBundle),
                        None,
                    ));
                    loaded.load_warnings.push(format!(
                        "failed to load committee reference pack artifact {path}: {err}"
                    ));
                }
            }
        }
        for path in &config.committee_benchmark_bundle_paths {
            match load_committee_benchmark_bundle(path) {
                Ok(Some(bundle)) => {
                    loaded.inventory_entries.push((
                        path.clone(),
                        Some(crate::experiment::CorePerformanceArtifactKind::CommitteeBenchmarkBundle),
                        serde_json::to_value(&bundle).ok(),
                    ));
                    loaded.benchmark_bundles.push(bundle);
                }
                Ok(None) => {
                    loaded.inventory_entries.push((
                        path.clone(),
                        Some(crate::experiment::CorePerformanceArtifactKind::CommitteeBenchmarkBundle),
                        None,
                    ));
                    loaded
                        .load_warnings
                        .push(format!("skipped committee benchmark artifact: {path}"));
                }
                Err(err) => {
                    loaded.inventory_entries.push((
                        path.clone(),
                        Some(crate::experiment::CorePerformanceArtifactKind::CommitteeBenchmarkBundle),
                        None,
                    ));
                    loaded.load_warnings.push(format!(
                        "failed to load committee benchmark artifact {path}: {err}"
                    ));
                }
            }
        }
        for path in &config.source_aware_benchmark_paths {
            match load_source_benchmark(path) {
                Ok(Some(report)) => {
                    loaded.inventory_entries.push((
                        path.clone(),
                        Some(crate::experiment::CorePerformanceArtifactKind::SourceAwareBenchmarkReport),
                        serde_json::to_value(&report).ok(),
                    ));
                    loaded.source_benchmarks.push(report);
                }
                Ok(None) => {
                    loaded.inventory_entries.push((
                        path.clone(),
                        Some(crate::experiment::CorePerformanceArtifactKind::SourceAwareBenchmarkReport),
                        None,
                    ));
                    loaded
                        .load_warnings
                        .push(format!("skipped source benchmark artifact: {path}"));
                }
                Err(err) => {
                    loaded.inventory_entries.push((
                        path.clone(),
                        Some(crate::experiment::CorePerformanceArtifactKind::SourceAwareBenchmarkReport),
                        None,
                    ));
                    loaded.load_warnings.push(format!(
                        "failed to load source benchmark artifact {path}: {err}"
                    ));
                }
            }
        }
        for path in &config.yahoo_research_report_paths {
            match load_yahoo_report(path) {
                Ok(Some(report)) => {
                    loaded.inventory_entries.push((
                        path.clone(),
                        Some(crate::experiment::CorePerformanceArtifactKind::YahooResearchEvidenceReport),
                        serde_json::to_value(&report).ok(),
                    ));
                    loaded.yahoo_reports.push(report);
                }
                Ok(None) => {
                    loaded.inventory_entries.push((
                        path.clone(),
                        Some(crate::experiment::CorePerformanceArtifactKind::YahooResearchEvidenceReport),
                        None,
                    ));
                    loaded
                        .load_warnings
                        .push(format!("skipped yahoo research artifact: {path}"));
                }
                Err(err) => {
                    loaded.inventory_entries.push((
                        path.clone(),
                        Some(crate::experiment::CorePerformanceArtifactKind::YahooResearchEvidenceReport),
                        None,
                    ));
                    loaded.load_warnings.push(format!(
                        "failed to load yahoo research artifact {path}: {err}"
                    ));
                }
            }
        }
        loaded
    }
}

fn build_signal_inputs(
    config: &CorePerformanceScorecardConfig,
    inventory: &CorePerformanceArtifactInventory,
    loaded: &LoadedCorePerformanceArtifacts,
) -> SignalQualityInputs {
    let mut inputs = SignalQualityInputs {
        controlled_only: inventory.controlled_only_count > 0
            && inventory.non_crypto_official_count == 0,
        require_official_for_usefulness_claim: config.require_official_for_usefulness_claim,
        ..SignalQualityInputs::default()
    };

    for report in &loaded.official_replications {
        inputs.evaluated_rows = inputs
            .evaluated_rows
            .max(report.row_injection_result.injected_rows.len());
        inputs.official_evaluated_rows = inputs.official_evaluated_rows.max(
            report
                .official_sufficiency_replication_report
                .non_crypto_official_row_count,
        );
        inputs.outcome_linked_rows = inputs.outcome_linked_rows.max(
            report
                .official_sufficiency_replication_report
                .outcome_link_count,
        );
        inputs.baseline_reference_rows = inputs.baseline_reference_rows.max(
            report
                .official_sufficiency_replication_report
                .baseline_reference_count,
        );
        inputs.no_trade_rows = inputs.no_trade_rows.max(
            report
                .official_sufficiency_replication_report
                .no_trade_counterfactual_count,
        );
    }

    for bundle in &loaded.outcome_coverage_bundles {
        inputs.evaluated_rows = inputs.evaluated_rows.max(bundle.coverage_report.total_rows);
        inputs.official_evaluated_rows = inputs
            .official_evaluated_rows
            .max(bundle.coverage_report.official_rows);
        inputs.outcome_linked_rows = inputs
            .outcome_linked_rows
            .max(bundle.coverage_report.outcome_linked_rows);
        inputs.baseline_reference_rows = inputs
            .baseline_reference_rows
            .max(bundle.coverage_report.baseline_linked_rows);
        inputs.external_reference_rows = inputs
            .external_reference_rows
            .max(bundle.coverage_report.external_linked_rows);
        inputs.committee_decision_rows = inputs
            .committee_decision_rows
            .max(bundle.performance_matrix.total_comparable_rows);
        inputs.baseline_action_rows = inputs
            .baseline_action_rows
            .max(bundle.coverage_report.baseline_linked_rows);
        inputs.no_trade_rows = inputs
            .no_trade_rows
            .max(bundle.coverage_report.no_trade_counterfactuals);
    }

    for report in benchmark_reports(loaded) {
        inputs.committee_decision_rows = inputs
            .committee_decision_rows
            .max(report.committee_benchmark_report.replay_report.record_count);
        inputs.baseline_action_rows = inputs.baseline_action_rows.max(
            report
                .outcome_linked_vs_baseline_report
                .baseline_action_counts
                .values()
                .sum(),
        );
        inputs.outcome_linked_rows = inputs
            .outcome_linked_rows
            .max(report.outcome_linked_vs_baseline_report.outcome_linked_rows);
        inputs.external_reference_rows = inputs.external_reference_rows.max(
            report
                .outcome_linked_vs_baseline_report
                .external_action_counts
                .as_ref()
                .map(|counts| counts.values().sum())
                .unwrap_or(0),
        );
        inputs.net_return_proxy = max_option(
            inputs.net_return_proxy,
            report
                .outcome_linked_vs_baseline_report
                .committee_net_return_proxy,
        );
        inputs.realized_edge_proxy = max_option(
            inputs.realized_edge_proxy,
            report
                .outcome_linked_vs_baseline_report
                .committee_vs_baseline_delta,
        );
    }

    for report in &loaded.source_benchmarks {
        inputs.brier_score = max_option(
            inputs.brier_score,
            report
                .official_summary
                .as_ref()
                .and_then(|summary| summary.avg_brier_score),
        );
        inputs.ece = max_option(
            inputs.ece,
            report
                .official_summary
                .as_ref()
                .and_then(|summary| summary.avg_expected_calibration_error),
        );
        inputs.net_return_proxy = max_option(
            inputs.net_return_proxy,
            report
                .official_summary
                .as_ref()
                .and_then(|summary| summary.avg_net_return_pct),
        );
    }

    if inventory.research_only_count > 0
        && inventory.non_crypto_official_count == 0
        && inventory.controlled_only_count == 0
        && inventory.fixture_only_count == 0
        && inventory.crypto_only_count == 0
    {
        inputs.research_only = true;
    }
    if inventory.fixture_only_count > 0
        && inventory.non_crypto_official_count == 0
        && inventory.controlled_only_count == 0
    {
        inputs.fixture_only = true;
    }
    if inventory.crypto_only_count > 0
        && inventory.non_crypto_official_count == 0
        && inventory.research_only_count == 0
        && inventory.fixture_only_count == 0
    {
        inputs.crypto_only = true;
    }
    inputs
}

fn build_committee_value_inputs(
    inventory: &CorePerformanceArtifactInventory,
    loaded: &LoadedCorePerformanceArtifacts,
) -> CommitteeValueAttributionInputs {
    let mut inputs = CommitteeValueAttributionInputs {
        diagnostic_only: inventory.non_crypto_official_count == 0,
        ..CommitteeValueAttributionInputs::default()
    };
    for report in benchmark_reports(loaded) {
        if report.outcome_linked_vs_baseline_report.comparable_rows >= inputs.comparable_rows {
            inputs.comparable_rows = report.outcome_linked_vs_baseline_report.comparable_rows;
            inputs.official_comparable_rows =
                report.outcome_linked_vs_baseline_report.comparable_rows;
            inputs.committee_action_counts = report
                .outcome_linked_vs_baseline_report
                .committee_final_action_counts
                .clone();
            inputs.baseline_action_counts = report
                .outcome_linked_vs_baseline_report
                .baseline_action_counts
                .clone();
            inputs.no_trade_baseline_counts = report
                .outcome_linked_vs_baseline_report
                .no_trade_baseline_counts
                .clone();
            inputs.external_action_counts = report
                .outcome_linked_vs_baseline_report
                .external_action_counts
                .clone();
            inputs.committee_vs_baseline_delta = report
                .outcome_linked_vs_baseline_report
                .committee_vs_baseline_delta;
            inputs.committee_vs_no_trade_delta = report
                .outcome_linked_vs_baseline_report
                .committee_vs_no_trade_delta;
            inputs.committee_vs_external_delta = report
                .outcome_linked_vs_baseline_report
                .committee_vs_external_delta;
            inputs.persona_contribution_summary = report
                .attribution_report
                .persona_contributions
                .iter()
                .map(|persona| {
                    (
                        persona.persona_id.clone(),
                        format!("{:.6}", persona.decision_influence_proxy),
                    )
                })
                .collect();
            inputs.chair_contribution_summary =
                report.attribution_report.chair_contribution_summary.clone();
            inputs.risk_contribution_summary = report
                .attribution_report
                .risk_governor_contribution_summary
                .clone();
            inputs.source_contribution_summary = report
                .attribution_report
                .source_contribution_summary
                .clone();
            inputs.chair_dominated = report.attribution_report.attribution_status
                == CommitteeAttributionStatus::ChairDominated;
            inputs.persona_dominated = report.attribution_report.attribution_status
                == CommitteeAttributionStatus::PersonaDominated;
            inputs.risk_dominated = report.attribution_report.attribution_status
                == CommitteeAttributionStatus::RiskDominated;
        }
    }
    inputs
}

fn build_risk_inputs(
    inventory: &CorePerformanceArtifactInventory,
    loaded: &LoadedCorePerformanceArtifacts,
    committee_value_report: &crate::league::CommitteeValueAttributionReport,
) -> RiskGovernorValueInputs {
    let mut inputs = RiskGovernorValueInputs {
        evidence_weak: inventory.non_crypto_official_count == 0
            || committee_value_report.comparable_rows == 0,
        diagnostic_only: inventory.non_crypto_official_count == 0,
        ..RiskGovernorValueInputs::default()
    };

    if let Some(replay) = best_replay_report(loaded) {
        inputs.total_decisions = replay.record_count;
        for record in &replay.records {
            match record.final_action {
                CommitteeFinalAction::PaperApprove => inputs.approved_count += 1,
                CommitteeFinalAction::PaperReduceSize
                | CommitteeFinalAction::HumanConfirmRequired => inputs.reduced_count += 1,
                CommitteeFinalAction::FinalNoTrade => inputs.no_trade_count += 1,
                CommitteeFinalAction::FinalDenied => inputs.denied_count += 1,
            }
            match record.risk_bridge_outcome.risk_decision.kind {
                RiskDecisionKind::EmergencyStop => {
                    inputs.emergency_stop_count += 1;
                    inputs.hard_veto_count += 1;
                }
                RiskDecisionKind::Cooldown => {
                    inputs.cooldown_count += 1;
                    inputs.hard_veto_count += 1;
                }
                RiskDecisionKind::Deny => {
                    let hard = record
                        .risk_bridge_outcome
                        .risk_decision
                        .reason_codes
                        .iter()
                        .any(|reason| {
                            matches!(
                                reason,
                                crate::core::ReasonCode::DailyLossGateBreached
                                    | crate::core::ReasonCode::ApiHealthGateBreached
                                    | crate::core::ReasonCode::DataQualityGateBreached
                            )
                        });
                    if hard {
                        inputs.hard_veto_count += 1;
                    }
                }
                RiskDecisionKind::ApprovePaper => {}
            }
        }
        inputs.soft_threshold_denial_count = inputs
            .denied_count
            .saturating_sub(inputs.hard_veto_count.min(inputs.denied_count));
    }

    if let Some((risk_count, avoided_loss, missed_gain)) = risk_counterfactual_metrics(loaded) {
        inputs.risk_denied_counterfactual_count = risk_count;
        inputs.avoided_loss_total = avoided_loss;
        inputs.missed_gain_total = missed_gain;
    }
    inputs
}

fn build_no_trade_inputs(
    inventory: &CorePerformanceArtifactInventory,
    loaded: &LoadedCorePerformanceArtifacts,
) -> NoTradeValueInputs {
    let mut inputs = NoTradeValueInputs {
        diagnostic_only: inventory.non_crypto_official_count == 0,
        ..NoTradeValueInputs::default()
    };
    if let Some(replay) = best_replay_report(loaded) {
        inputs.no_trade_decisions = replay
            .records
            .iter()
            .filter(|record| record.final_action == CommitteeFinalAction::FinalNoTrade)
            .count();
    }
    if let Some((count, avoided_loss, missed_gain)) = no_trade_counterfactual_metrics(loaded) {
        inputs.no_trade_counterfactuals = count;
        inputs.avoided_loss_value = avoided_loss;
        inputs.missed_gain_value = missed_gain;
    }
    inputs
}

fn build_latency_budget_report(
    config: &CorePerformanceScorecardConfig,
    inventory: &CorePerformanceArtifactInventory,
    signal_quality_report: &crate::experiment::SignalQualityReport,
    committee_value_report: &crate::league::CommitteeValueAttributionReport,
    risk_report: &crate::risk::RiskGovernorValueReport,
    no_trade_report: &crate::risk::NoTradeValueReport,
) -> CoreLatencyBudgetReport {
    let artifacts = inventory
        .descriptors
        .iter()
        .map(|descriptor| ArtifactSize {
            path: descriptor.path.clone(),
            bytes: fs::metadata(&descriptor.path)
                .map(|metadata| metadata.len() as usize)
                .unwrap_or_default(),
        })
        .collect::<Vec<_>>();
    let report_bytes = signal_quality_report.to_text().len()
        + committee_value_report.to_text().len()
        + risk_report.to_text().len()
        + no_trade_report.to_text().len();
    let row_count = signal_quality_report
        .evaluated_rows
        .max(signal_quality_report.outcome_linked_rows)
        .max(committee_value_report.comparable_rows)
        .max(risk_report.total_decisions)
        .max(no_trade_report.no_trade_decisions);
    let latency_config = CoreLatencyBudgetConfig {
        max_scorecard_artifacts: config.max_artifacts,
        max_rows: config.max_rows,
        max_report_bytes: config.max_bytes,
        max_artifact_bytes: config.max_bytes,
        max_decision_path_steps: 12,
        target_decision_latency_ms: Some(80),
        reason_codes: vec![crate::core::ReasonCode::DeterministicPath],
    };
    build_core_latency_budget_report(&latency_config, &artifacts, row_count, report_bytes, 11)
}

fn build_bottleneck_inputs(
    inventory: &CorePerformanceArtifactInventory,
    loaded: &LoadedCorePerformanceArtifacts,
    signal_quality_report: &crate::experiment::SignalQualityReport,
    committee_value_report: &crate::league::CommitteeValueAttributionReport,
    risk_report: &crate::risk::RiskGovernorValueReport,
    no_trade_report: &crate::risk::NoTradeValueReport,
    latency_budget_report: &CoreLatencyBudgetReport,
) -> CoreBottleneckInputs {
    let provider_auth_missing = loaded.official_replications.iter().any(|report| {
        matches!(
            report.final_status,
            crate::league::OfficialEvidenceReplicationFinalStatus::MissingOfficialAuth
        )
    });
    let official_data_missing = inventory.non_crypto_official_count == 0
        && inventory.controlled_only_count == 0
        && inventory.research_only_count == 0
        && inventory.fixture_only_count == 0
        && inventory.crypto_only_count == 0;
    let official_candles_missing = loaded.official_replications.iter().any(|report| {
        matches!(
            report.final_status,
            crate::league::OfficialEvidenceReplicationFinalStatus::MissingOfficialCandles
        )
    });
    CoreBottleneckInputs {
        provider_auth_missing,
        official_data_missing,
        official_candles_missing,
        outcome_links_missing: signal_quality_report.outcome_linked_rows == 0,
        baseline_references_missing: signal_quality_report.baseline_reference_rows == 0,
        no_trade_counterfactuals_missing: no_trade_report.no_trade_decisions > 0
            && no_trade_report.no_trade_counterfactuals == 0,
        risk_denied_counterfactuals_missing: risk_report.denied_count > 0
            && risk_report.risk_denied_counterfactual_count == 0,
        poor_calibration: signal_quality_report.signal_quality_status
            == crate::experiment::SignalQualityStatus::PoorCalibration,
        risk_overblocking: risk_report.overblocking_suspected,
        risk_underblocking: risk_report.underblocking_suspected,
        chair_dominated: committee_value_report.attribution_status
            == CommitteeValueAttributionStatus::ChairDominated,
        persona_scoring_weak: committee_value_report.attribution_status
            == CommitteeValueAttributionStatus::PersonaDominated,
        signal_model_weak: committee_value_report.attribution_status
            == CommitteeValueAttributionStatus::CommitteeNoBetterThanBaseline
            && committee_value_report.comparable_rows > 0,
        scenario_materialization_weak: committee_value_report.comparable_rows == 0,
        storage_budget_exceeded: matches!(
            latency_budget_report.budget_status,
            CoreLatencyBudgetStatus::StorageBudgetExceeded
                | CoreLatencyBudgetStatus::TooManyArtifacts
                | CoreLatencyBudgetStatus::TooManyRows
        ),
        latency_budget_exceeded: latency_budget_report.budget_status
            == CoreLatencyBudgetStatus::LatencyBudgetExceeded,
        evidence_too_weak: inventory.non_crypto_official_count == 0
            && inventory.controlled_only_count == 0
            && signal_quality_report.outcome_linked_rows == 0,
        reason_codes: vec![crate::core::ReasonCode::DeterministicPath],
    }
}

fn build_regression_report(
    config: &CorePerformanceScorecardConfig,
    scorecard: &CorePerformanceScorecard,
) -> Result<Option<CorePerformanceRegressionReport>, String> {
    let current_summary = summary_from_scorecard(scorecard);
    let previous_summary = config
        .previous_scorecard_paths
        .iter()
        .find_map(|path| load_previous_summary(path).ok());
    if previous_summary.is_none() && config.previous_scorecard_paths.is_empty() {
        return Ok(None);
    }
    let regression_config = CorePerformanceRegressionConfig {
        previous_scorecard_path: None,
        current_scorecard_path: None,
        ..CorePerformanceRegressionConfig::default()
    };
    Ok(Some(build_core_performance_regression_report(
        &regression_config,
        previous_summary,
        current_summary,
    )))
}

fn load_previous_summary(path: &str) -> Result<CorePerformanceRegressionSummary, String> {
    let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
    if let Ok(summary) = serde_json::from_str::<CorePerformanceRegressionSummary>(&text) {
        return Ok(summary);
    }
    if let Ok(scorecard) = serde_json::from_str::<CorePerformanceScorecard>(&text) {
        return Ok(summary_from_scorecard(&scorecard));
    }
    if let Ok(bundle) = serde_json::from_str::<CorePerformanceScorecardBundle>(&text) {
        return Ok(summary_from_scorecard(&bundle.scorecard));
    }
    let scorecard = CorePerformanceScorecard::from_json_path(Path::new(path))?;
    Ok(summary_from_scorecard(&scorecard))
}

fn benchmark_reports<'a>(
    loaded: &'a LoadedCorePerformanceArtifacts,
) -> Vec<&'a CommitteeOfficialBenchmarkReport> {
    let mut reports = loaded.official_benchmarks.iter().collect::<Vec<_>>();
    reports.extend(
        loaded
            .official_replications
            .iter()
            .filter_map(|report| report.official_committee_benchmark_report.as_ref()),
    );
    reports
}

fn best_replay_report(
    loaded: &LoadedCorePerformanceArtifacts,
) -> Option<&crate::league::CommitteeReplayReport> {
    let direct = benchmark_reports(loaded)
        .into_iter()
        .map(|report| &report.committee_benchmark_report.replay_report)
        .max_by_key(|report| report.record_count);
    let bundle = loaded
        .benchmark_bundles
        .iter()
        .map(|bundle| &bundle.replay_report)
        .max_by_key(|report| report.record_count);
    direct.or(bundle)
}

fn risk_counterfactual_metrics(
    loaded: &LoadedCorePerformanceArtifacts,
) -> Option<(usize, f64, f64)> {
    let audit = best_counterfactual_audit(loaded)?;
    let records = audit
        .records
        .iter()
        .filter(|record| {
            record.counterfactual_type == CommitteeCounterfactualType::RiskDenied && record.built()
        })
        .collect::<Vec<_>>();
    if records.is_empty() {
        return Some((0, 0.0, 0.0));
    }
    Some((
        records.len(),
        records
            .iter()
            .filter_map(|record| record.avoided_loss_value)
            .sum(),
        records
            .iter()
            .filter_map(|record| record.missed_gain_value)
            .sum(),
    ))
}

fn no_trade_counterfactual_metrics(
    loaded: &LoadedCorePerformanceArtifacts,
) -> Option<(usize, f64, f64)> {
    let audit = best_counterfactual_audit(loaded)?;
    let records = audit
        .records
        .iter()
        .filter(|record| {
            record.counterfactual_type == CommitteeCounterfactualType::NoTrade && record.built()
        })
        .collect::<Vec<_>>();
    if records.is_empty() {
        return Some((0, 0.0, 0.0));
    }
    Some((
        records.len(),
        records
            .iter()
            .filter_map(|record| record.avoided_loss_value)
            .sum(),
        records
            .iter()
            .filter_map(|record| record.missed_gain_value)
            .sum(),
    ))
}

fn best_counterfactual_audit(
    loaded: &LoadedCorePerformanceArtifacts,
) -> Option<&CommitteeCounterfactualAuditReport> {
    loaded
        .outcome_coverage_bundles
        .iter()
        .filter_map(|bundle| bundle.counterfactual_audit_report.as_ref())
        .max_by_key(|report| report.built_count)
}

fn load_core_check(path: &str) -> Result<Option<CoreReadinessReport>, String> {
    if path.ends_with(".toml") {
        let config = CoreCheckConfig::from_toml_path(Path::new(path))?;
        CoreCheckRunner::default().run(&config).map(Some)
    } else {
        parse_json_file(path)
    }
}

fn load_official_replication(
    path: &str,
) -> Result<Option<OfficialEvidenceReplicationReport>, String> {
    if path.ends_with(".toml") {
        let config = OfficialEvidenceReplicationConfig::from_toml_path(Path::new(path))?;
        OfficialEvidenceReplicationRunner::default()
            .run(&config)
            .map(Some)
    } else {
        let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
        if let Ok(bundle) = serde_json::from_str::<OfficialEvidenceReplicationBundle>(&text) {
            Ok(Some(bundle.replication_report))
        } else {
            Ok(serde_json::from_str::<OfficialEvidenceReplicationReport>(&text).ok())
        }
    }
}

fn load_official_benchmark(path: &str) -> Result<Option<CommitteeOfficialBenchmarkReport>, String> {
    if path.ends_with(".toml") {
        let config = CommitteeOfficialBenchmarkConfig::from_toml_path(Path::new(path))?;
        CommitteeOfficialBenchmarkRunner::default()
            .run(&config)
            .map(Some)
    } else {
        parse_json_file(path)
    }
}

fn load_outcome_coverage(path: &str) -> Result<Option<CommitteeOutcomeCoverageBundle>, String> {
    if path.ends_with(".toml") {
        let config = CommitteeOutcomeCoverageConfig::from_toml_path(Path::new(path))?;
        CommitteeOutcomeCoverageRunner::default()
            .run(&config)
            .map(Some)
    } else {
        parse_json_file(path)
    }
}

fn load_reference_pack(path: &str) -> Result<Option<CommitteeReferencePackBundle>, String> {
    if path.ends_with(".toml") {
        let config = CommitteeReferencePackConfig::from_toml_path(Path::new(path))?;
        CommitteeReferencePackRunner::default()
            .run(&config)
            .map(Some)
    } else {
        parse_json_file(path)
    }
}

fn load_committee_benchmark_bundle(path: &str) -> Result<Option<CommitteeBenchmarkBundle>, String> {
    if path.ends_with(".toml") {
        let config = CommitteeBenchmarkConfig::from_toml_path(Path::new(path))?;
        CommitteeBenchmarkRunner::default().run(&config).map(Some)
    } else {
        parse_json_file(path)
    }
}

fn load_source_benchmark(path: &str) -> Result<Option<SourceAwareBenchmarkReport>, String> {
    if path.ends_with(".toml") {
        let config = SourceAwareBenchmarkConfig::from_toml_path(Path::new(path))?;
        SourceAwareBenchmarkRunner::default().run(&config).map(Some)
    } else {
        parse_json_file(path)
    }
}

fn load_yahoo_report(path: &str) -> Result<Option<YahooResearchEvidenceReport>, String> {
    if path.ends_with(".toml") {
        let config = YahooResearchEvidenceConfig::from_toml_path(Path::new(path))?;
        YahooResearchEvidenceRunner::default()
            .run(&config)
            .map(Some)
    } else {
        parse_json_file(path)
    }
}

fn parse_json_file<T: DeserializeOwned>(path: &str) -> Result<Option<T>, String> {
    let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
    Ok(serde_json::from_str(&text).ok())
}

fn max_option(current: Option<f64>, candidate: Option<f64>) -> Option<f64> {
    match (current, candidate) {
        (Some(left), Some(right)) => Some(left.max(right)),
        (Some(left), None) => Some(left),
        (None, Some(right)) => Some(right),
        (None, None) => None,
    }
}

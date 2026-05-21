use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use serde::de::DeserializeOwned;

use crate::core::{ReasonCode, stable_reason_codes};
use crate::data::ProviderMarket;
use crate::experiment::{
    CorePerformanceFinalStatus, CorePerformanceScorecard, CorePerformanceScorecardConfig,
    CorePerformanceScorecardRunner, SourceAwareBenchmarkConfig, SourceAwareBenchmarkReport,
    SourceAwareBenchmarkRunner, YahooResearchEvidenceConfig, YahooResearchEvidenceReport,
    YahooResearchEvidenceRunner,
};

use super::committee_benchmark::{CommitteeBenchmarkConfig, CommitteeBenchmarkRunner};
use super::committee_benchmark_bundle::CommitteeBenchmarkBundle;
use super::committee_counterfactual_builder::CommitteeCounterfactualRecord;
use super::committee_official_benchmark::{
    CommitteeOfficialBenchmarkConfig, CommitteeOfficialBenchmarkReport,
    CommitteeOfficialBenchmarkRunner,
};
use super::committee_outcome_coverage::{CommitteeOutcomeCoverageConfig, OutcomeCoverageCell};
use super::committee_outcome_coverage_bundle::CommitteeOutcomeCoverageBundle;
use super::committee_outcome_linker::OutcomeLinkedCommitteeScenarioRow;
use super::committee_outcome_reference::CommitteeBaselineAction;
use super::committee_reference_pack::{
    CommitteeReferencePackConfig, GeneratedCommitteeReferencePack,
};
use super::committee_reference_pack_bundle::CommitteeReferencePackBundle;
use super::committee_reference_pack_runner::CommitteeReferencePackRunner;
use super::committee_replay::CommitteeReplayRecord;
use super::committee_risk_bridge::CommitteeFinalAction;
use super::comparable_committee_evidence::{
    ComparableCommitteeEvidenceBundle, ComparableCommitteeEvidenceConfig,
    ComparableCommitteeEvidenceRow, ComparableEvidenceSourceClass, infer_market_from_symbol,
};
use super::official_committee_benchmark_bundle::CommitteeOfficialBenchmarkBundle;
use super::official_evidence_replication::{
    OfficialEvidenceReplicationConfig, OfficialEvidenceReplicationReport,
    OfficialEvidenceReplicationRunner,
};
use super::official_replication_bundle::OfficialEvidenceReplicationBundle;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ComparableEvidenceBuilder;

impl ComparableEvidenceBuilder {
    pub fn build(
        &self,
        config: &ComparableCommitteeEvidenceConfig,
    ) -> Result<ComparableCommitteeEvidenceBundle, String> {
        config.validate()?;
        let rows = self.load_rows(config)?;
        Ok(self.finalize_bundle(config, rows))
    }

    pub fn finalize_bundle(
        &self,
        config: &ComparableCommitteeEvidenceConfig,
        rows: Vec<ComparableCommitteeEvidenceRow>,
    ) -> ComparableCommitteeEvidenceBundle {
        let deduped = dedupe_rows(config, rows);
        let bounded = bound_rows(config, deduped);
        ComparableCommitteeEvidenceBundle::from_rows(config, bounded)
    }

    pub fn merge_bundle(
        &self,
        config: &ComparableCommitteeEvidenceConfig,
        base: &ComparableCommitteeEvidenceBundle,
        extra_rows: Vec<ComparableCommitteeEvidenceRow>,
    ) -> ComparableCommitteeEvidenceBundle {
        let mut rows = base.rows.clone();
        rows.extend(extra_rows);
        self.finalize_bundle(config, rows)
    }

    pub fn load_rows(
        &self,
        config: &ComparableCommitteeEvidenceConfig,
    ) -> Result<Vec<ComparableCommitteeEvidenceRow>, String> {
        let mut rows = Vec::new();
        for path in &config.official_replication_report_paths {
            rows.extend(load_official_replication_rows(path)?);
        }
        for path in &config.official_committee_benchmark_paths {
            rows.extend(load_official_benchmark_rows(path)?);
        }
        for path in &config.reference_pack_bundle_paths {
            rows.extend(load_reference_pack_rows(path)?);
        }
        for path in &config.outcome_coverage_bundle_paths {
            rows.extend(load_outcome_coverage_rows(path)?);
        }
        for path in &config.committee_benchmark_bundle_paths {
            rows.extend(load_committee_benchmark_rows(path)?);
        }
        for path in &config.source_aware_benchmark_paths {
            rows.extend(load_source_benchmark_rows(path)?);
        }
        for path in &config.yahoo_research_report_paths {
            rows.extend(load_yahoo_report_rows(path)?);
        }
        for path in &config.core_performance_scorecard_paths {
            rows.extend(load_scorecard_rows(path)?);
        }
        Ok(rows)
    }
}

fn load_official_replication_rows(
    path: &str,
) -> Result<Vec<ComparableCommitteeEvidenceRow>, String> {
    if path.ends_with(".toml") {
        let config = OfficialEvidenceReplicationConfig::from_toml_path(Path::new(path))?;
        let bundle = OfficialEvidenceReplicationRunner::default().run_bundle(&config)?;
        return Ok(rows_from_official_replication_bundle(&bundle, path));
    }
    let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
    if let Ok(bundle) = serde_json::from_str::<OfficialEvidenceReplicationBundle>(&text) {
        return Ok(rows_from_official_replication_bundle(&bundle, path));
    }
    let report = serde_json::from_str::<OfficialEvidenceReplicationReport>(&text)
        .map_err(|err| err.to_string())?;
    Ok(rows_from_official_replication_report(&report, path))
}

fn load_official_benchmark_rows(path: &str) -> Result<Vec<ComparableCommitteeEvidenceRow>, String> {
    if path.ends_with(".toml") {
        let config = CommitteeOfficialBenchmarkConfig::from_toml_path(Path::new(path))?;
        let bundle = CommitteeOfficialBenchmarkRunner::default().run_bundle(&config)?;
        return Ok(rows_from_official_benchmark_bundle(&bundle, path));
    }
    let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
    if let Ok(bundle) = serde_json::from_str::<CommitteeOfficialBenchmarkBundle>(&text) {
        return Ok(rows_from_official_benchmark_bundle(&bundle, path));
    }
    let report = serde_json::from_str::<CommitteeOfficialBenchmarkReport>(&text)
        .map_err(|err| err.to_string())?;
    Ok(rows_from_official_benchmark_report(&report, path))
}

fn load_reference_pack_rows(path: &str) -> Result<Vec<ComparableCommitteeEvidenceRow>, String> {
    if path.ends_with(".toml") {
        let config = CommitteeReferencePackConfig::from_toml_path(Path::new(path))?;
        let bundle = CommitteeReferencePackRunner::default().run(&config)?;
        return Ok(rows_from_reference_pack_bundle(&bundle, path));
    }
    let bundle = parse_json_file::<CommitteeReferencePackBundle>(path)?
        .ok_or_else(|| format!("failed to parse reference pack bundle {path}"))?;
    Ok(rows_from_reference_pack_bundle(&bundle, path))
}

fn load_outcome_coverage_rows(path: &str) -> Result<Vec<ComparableCommitteeEvidenceRow>, String> {
    if path.ends_with(".toml") {
        let config = CommitteeOutcomeCoverageConfig::from_toml_path(Path::new(path))?;
        let bundle =
            super::committee_outcome_coverage_runner::CommitteeOutcomeCoverageRunner::default()
                .run(&config)?;
        return Ok(rows_from_outcome_coverage_bundle(&bundle, path));
    }
    let bundle = parse_json_file::<CommitteeOutcomeCoverageBundle>(path)?
        .ok_or_else(|| format!("failed to parse outcome coverage bundle {path}"))?;
    Ok(rows_from_outcome_coverage_bundle(&bundle, path))
}

fn load_committee_benchmark_rows(
    path: &str,
) -> Result<Vec<ComparableCommitteeEvidenceRow>, String> {
    if path.ends_with(".toml") {
        let config = CommitteeBenchmarkConfig::from_toml_path(Path::new(path))?;
        let bundle = CommitteeBenchmarkRunner::default().run(&config)?;
        return Ok(rows_from_committee_benchmark_bundle(&bundle, path));
    }
    let bundle = parse_json_file::<CommitteeBenchmarkBundle>(path)?
        .ok_or_else(|| format!("failed to parse committee benchmark bundle {path}"))?;
    Ok(rows_from_committee_benchmark_bundle(&bundle, path))
}

fn load_source_benchmark_rows(path: &str) -> Result<Vec<ComparableCommitteeEvidenceRow>, String> {
    let report = if path.ends_with(".toml") {
        let config = SourceAwareBenchmarkConfig::from_toml_path(Path::new(path))?;
        SourceAwareBenchmarkRunner::default().run(&config)?
    } else {
        parse_json_file::<SourceAwareBenchmarkReport>(path)?
            .ok_or_else(|| format!("failed to parse source benchmark report {path}"))?
    };
    Ok(rows_from_source_benchmark_report(&report, path))
}

fn load_yahoo_report_rows(path: &str) -> Result<Vec<ComparableCommitteeEvidenceRow>, String> {
    let report = if path.ends_with(".toml") {
        let config = YahooResearchEvidenceConfig::from_toml_path(Path::new(path))?;
        YahooResearchEvidenceRunner::default().run(&config)?
    } else {
        parse_json_file::<YahooResearchEvidenceReport>(path)?
            .ok_or_else(|| format!("failed to parse yahoo report {path}"))?
    };
    Ok(rows_from_yahoo_report(&report, path))
}

fn load_scorecard_rows(path: &str) -> Result<Vec<ComparableCommitteeEvidenceRow>, String> {
    let scorecard = if path.ends_with(".toml") {
        let config = CorePerformanceScorecardConfig::from_toml_path(Path::new(path))?;
        CorePerformanceScorecardRunner::default()
            .run(&config)?
            .scorecard
    } else {
        let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
        if let Ok(bundle) =
            serde_json::from_str::<crate::experiment::CorePerformanceScorecardBundle>(&text)
        {
            bundle.scorecard
        } else {
            serde_json::from_str::<CorePerformanceScorecard>(&text)
                .map_err(|err| err.to_string())?
        }
    };
    Ok(rows_from_scorecard(&scorecard, path))
}

fn rows_from_official_replication_bundle(
    bundle: &OfficialEvidenceReplicationBundle,
    source_path: &str,
) -> Vec<ComparableCommitteeEvidenceRow> {
    let mut rows = Vec::new();
    if let Some(reference_replication) = &bundle.reference_replication {
        if let Some(pack) = &reference_replication.generated_reference_pack {
            rows.extend(rows_from_reference_pack(
                pack,
                &format!("official-reference:{source_path}"),
            ));
        }
    }
    if rows.is_empty() {
        if let Some(pack) = &bundle.injected_scenario_pack {
            rows.extend(pack.rows.iter().map(|row| {
                let mut comparable = ComparableCommitteeEvidenceRow::from_scenario_row(row);
                comparable.row_id =
                    format!("official-injection:{source_path}:{}", comparable.row_id);
                comparable.reason_codes = stable_reason_codes(
                    &comparable
                        .reason_codes
                        .into_iter()
                        .chain([ReasonCode::OfficialEvidenceReplicationBuilt])
                        .collect::<Vec<_>>(),
                );
                comparable
            }));
        }
    }
    if rows.is_empty() {
        rows.extend(rows_from_official_replication_report(
            &bundle.replication_report,
            source_path,
        ));
    }
    rows
}

fn rows_from_official_replication_report(
    report: &OfficialEvidenceReplicationReport,
    source_path: &str,
) -> Vec<ComparableCommitteeEvidenceRow> {
    let source_class = if report
        .official_sufficiency_replication_report
        .non_crypto_official_row_count
        > 0
    {
        ComparableEvidenceSourceClass::OfficialNonCrypto
    } else if report
        .official_sufficiency_replication_report
        .crypto_only_ratio
        > 0.0
    {
        ComparableEvidenceSourceClass::OfficialCryptoOnly
    } else if matches!(
        report.final_status,
        super::official_evidence_replication::OfficialEvidenceReplicationFinalStatus::ControlledOnly
    ) {
        ComparableEvidenceSourceClass::ControlledDiagnostic
    } else {
        ComparableEvidenceSourceClass::Unknown
    };
    vec![ComparableCommitteeEvidenceRow {
        row_id: format!("official-replication-summary:{}", report.replication_id),
        symbol: report
            .artifact_inventory
            .descriptors
            .iter()
            .find_map(|descriptor| descriptor.symbol.clone())
            .unwrap_or_else(|| "SUMMARY".to_string()),
        market: report
            .artifact_inventory
            .descriptors
            .iter()
            .find_map(|descriptor| descriptor.market)
            .unwrap_or(ProviderMarket::USEquity),
        timeframe: "summary".to_string(),
        horizon_bars: 0,
        timestamp_ms: 0,
        source_kind: format!("OfficialEvidenceReplication:{source_path}"),
        source_class,
        scenario_row_id: None,
        committee_decision_id: None,
        committee_final_action: "SummaryOnly".to_string(),
        chair_decision: None,
        risk_governor_decision: None,
        baseline_action: None,
        external_action: None,
        no_trade_baseline_action: "NoTrade".to_string(),
        outcome_label: Some(format!("{:?}", report.final_status)),
        net_return_pct: None,
        cost_bps: 0.0,
        slippage_bps: 0.0,
        committee_vs_baseline_delta: None,
        committee_vs_notrade_delta: None,
        risk_denied_value_proxy: None,
        no_trade_value_proxy: None,
        outcome_reference_available: report
            .official_sufficiency_replication_report
            .outcome_link_count
            > 0,
        baseline_reference_available: report
            .official_sufficiency_replication_report
            .baseline_reference_count
            > 0,
        no_trade_counterfactual_available: report
            .official_sufficiency_replication_report
            .no_trade_counterfactual_count
            > 0,
        risk_denied_counterfactual_available: report
            .official_sufficiency_replication_report
            .risk_denied_counterfactual_count
            > 0,
        external_reference_available: false,
        row_level: false,
        summary_derived: true,
        no_lookahead_safe: true,
        official_readiness_eligible: false,
        diagnostic_only: source_class != ComparableEvidenceSourceClass::OfficialNonCrypto,
        candle_coverage_available: false,
        matched_candle_series_id: None,
        candle_match_status: None,
        candle_official_ready_match: false,
        candle_benchmark_ready_match: false,
        candle_diagnostic_only: false,
        reason_codes: stable_reason_codes(
            &report
                .reason_codes
                .iter()
                .cloned()
                .chain([
                    ReasonCode::SummaryDerived,
                    ReasonCode::OfficialEvidenceReplicationBuilt,
                ])
                .collect::<Vec<_>>(),
        ),
    }]
}

fn rows_from_official_benchmark_bundle(
    bundle: &CommitteeOfficialBenchmarkBundle,
    source_path: &str,
) -> Vec<ComparableCommitteeEvidenceRow> {
    let replay_by_row = bundle
        .committee_benchmark_report
        .replay_report
        .records
        .iter()
        .map(|record| (record.scenario_row.scenario_row_id.clone(), record))
        .collect::<BTreeMap<_, _>>();
    bundle
        .outcome_linked_pack
        .linked_rows
        .iter()
        .map(|linked| {
            row_from_linked_row(
                linked,
                replay_by_row
                    .get(&linked.scenario_row.scenario_row_id)
                    .copied(),
                None,
                None,
                &format!("official-benchmark:{source_path}"),
            )
        })
        .collect()
}

fn rows_from_official_benchmark_report(
    report: &CommitteeOfficialBenchmarkReport,
    source_path: &str,
) -> Vec<ComparableCommitteeEvidenceRow> {
    let comparable_rows = report.outcome_linked_vs_baseline_report.comparable_rows;
    let source_class = match report.final_status {
        super::committee_official_benchmark::CommitteeOfficialBenchmarkFinalStatus::ResearchOnly => {
            ComparableEvidenceSourceClass::YFinanceResearch
        }
        super::committee_official_benchmark::CommitteeOfficialBenchmarkFinalStatus::FixtureOnly => {
            ComparableEvidenceSourceClass::FixtureArchitectureTest
        }
        super::committee_official_benchmark::CommitteeOfficialBenchmarkFinalStatus::CryptoOnly => {
            ComparableEvidenceSourceClass::OfficialCryptoOnly
        }
        _ => ComparableEvidenceSourceClass::OfficialNonCrypto,
    };
    vec![ComparableCommitteeEvidenceRow {
        row_id: format!("official-benchmark-summary:{}", report.benchmark_id),
        symbol: "SUMMARY".to_string(),
        market: ProviderMarket::USEquity,
        timeframe: "summary".to_string(),
        horizon_bars: 0,
        timestamp_ms: 0,
        source_kind: format!("CommitteeOfficialBenchmark:{source_path}"),
        source_class,
        scenario_row_id: None,
        committee_decision_id: None,
        committee_final_action: "SummaryOnly".to_string(),
        chair_decision: None,
        risk_governor_decision: None,
        baseline_action: None,
        external_action: None,
        no_trade_baseline_action: "NoTrade".to_string(),
        outcome_label: Some(format!("{:?}", report.final_status)),
        net_return_pct: report
            .outcome_linked_vs_baseline_report
            .committee_net_return_proxy,
        cost_bps: 0.0,
        slippage_bps: 0.0,
        committee_vs_baseline_delta: report
            .outcome_linked_vs_baseline_report
            .committee_vs_baseline_delta,
        committee_vs_notrade_delta: report
            .outcome_linked_vs_baseline_report
            .committee_vs_no_trade_delta,
        risk_denied_value_proxy: Some(
            report
                .outcome_linked_vs_baseline_report
                .risk_denied_value_proxy,
        ),
        no_trade_value_proxy: Some(
            report
                .outcome_linked_vs_baseline_report
                .no_trade_value_proxy,
        ),
        outcome_reference_available: report.outcome_linked_vs_baseline_report.outcome_linked_rows
            > 0,
        baseline_reference_available: !report
            .outcome_linked_vs_baseline_report
            .baseline_action_counts
            .is_empty(),
        no_trade_counterfactual_available: report
            .official_evidence_readiness_report
            .no_trade_counterfactual_count
            >= 1,
        risk_denied_counterfactual_available: report
            .official_evidence_readiness_report
            .risk_denial_counterfactual_count
            >= 1,
        external_reference_available: report
            .outcome_linked_vs_baseline_report
            .external_action_counts
            .as_ref()
            .is_some_and(|counts| !counts.is_empty()),
        row_level: false,
        summary_derived: true,
        no_lookahead_safe: true,
        official_readiness_eligible: false,
        diagnostic_only: comparable_rows == 0
            || source_class != ComparableEvidenceSourceClass::OfficialNonCrypto,
        candle_coverage_available: false,
        matched_candle_series_id: None,
        candle_match_status: None,
        candle_official_ready_match: false,
        candle_benchmark_ready_match: false,
        candle_diagnostic_only: false,
        reason_codes: stable_reason_codes(
            &report
                .reason_codes
                .iter()
                .cloned()
                .chain([ReasonCode::SummaryDerived])
                .collect::<Vec<_>>(),
        ),
    }]
}

fn rows_from_reference_pack_bundle(
    bundle: &CommitteeReferencePackBundle,
    source_path: &str,
) -> Vec<ComparableCommitteeEvidenceRow> {
    rows_from_reference_pack(
        &bundle.reference_pack,
        &format!("reference-pack:{source_path}"),
    )
}

fn rows_from_reference_pack(
    pack: &GeneratedCommitteeReferencePack,
    prefix: &str,
) -> Vec<ComparableCommitteeEvidenceRow> {
    let counterfactuals = pack.generated_references.iter().fold(
        BTreeMap::<String, Vec<&super::committee_reference_pack::GeneratedCommitteeReference>>::new(
        ),
        |mut acc, reference| {
            acc.entry(reference.scenario_row_id.clone())
                .or_default()
                .push(reference);
            acc
        },
    );
    pack.scenario_rows
        .iter()
        .map(|row| {
            let generated = counterfactuals.get(&row.scenario_row_id).cloned().unwrap_or_default();
            let outcome_reference = generated
                .iter()
                .find(|item| item.reference_kind == super::committee_reference_pack::GeneratedReferenceKind::TripleBarrierOutcome)
                .and_then(|item| item.outcome_reference.as_ref());
            let baseline_reference = generated
                .iter()
                .find(|item| item.reference_kind == super::committee_reference_pack::GeneratedReferenceKind::BaselineAction)
                .and_then(|item| item.baseline_reference.as_ref());
            let external_reference = generated
                .iter()
                .find(|item| item.reference_kind == super::committee_reference_pack::GeneratedReferenceKind::ExternalPredictionAction)
                .and_then(|item| item.external_reference.as_ref());
            let no_trade_record = generated
                .iter()
                .find(|item| item.reference_kind == super::committee_reference_pack::GeneratedReferenceKind::NoTradeCounterfactual)
                .and_then(|item| item.no_trade_counterfactual.as_ref());
            let risk_record = generated
                .iter()
                .find(|item| item.reference_kind == super::committee_reference_pack::GeneratedReferenceKind::RiskDeniedCounterfactual)
                .and_then(|item| item.risk_denied_counterfactual.as_ref());
            let mut comparable = ComparableCommitteeEvidenceRow::from_scenario_row(row);
            comparable.row_id = format!("{prefix}:{}", row.scenario_row_id);
            comparable.outcome_label = outcome_reference.map(|item| format!("{:?}", item.triple_barrier_label));
            comparable.net_return_pct = outcome_reference.and_then(|item| item.net_return_pct);
            comparable.cost_bps = outcome_reference.map(|item| item.cost_bps).unwrap_or_default();
            comparable.slippage_bps = outcome_reference
                .map(|item| item.slippage_bps)
                .unwrap_or_default();
            comparable.baseline_action = baseline_reference
                .map(|item| item.baseline_action.as_summary_str().to_string())
                .or_else(|| comparable.baseline_action.clone());
            comparable.external_action = external_reference
                .and_then(|item| item.external_action.clone())
                .or_else(|| comparable.external_action.clone());
            comparable.outcome_reference_available = outcome_reference.is_some();
            comparable.baseline_reference_available = baseline_reference.is_some() || comparable.baseline_reference_available;
            comparable.no_trade_counterfactual_available = no_trade_record.is_some() || comparable.no_trade_counterfactual_available;
            comparable.risk_denied_counterfactual_available = risk_record.is_some() || comparable.risk_denied_counterfactual_available;
            comparable.external_reference_available = external_reference.is_some() || comparable.external_reference_available;
            comparable.no_lookahead_safe = outcome_reference
                .map(|item| item.no_lookahead_safe)
                .or_else(|| no_trade_record.map(|item| item.no_lookahead_safe))
                .or_else(|| risk_record.map(|item| item.no_lookahead_safe))
                .unwrap_or(comparable.no_lookahead_safe);
            comparable.diagnostic_only = comparable.diagnostic_only
                || no_trade_record.is_some_and(|item| item.diagnostic_only)
                || risk_record.is_some_and(|item| item.diagnostic_only);
            comparable.official_readiness_eligible = comparable.official_readiness_eligible
                && !comparable.diagnostic_only
                && outcome_reference.is_some();
            comparable.no_trade_value_proxy = no_trade_record
                .and_then(|item| item.avoided_loss_value)
                .or_else(|| {
                    no_trade_record
                        .and_then(|item| item.net_return_pct)
                        .map(f64::abs)
                });
            comparable.risk_denied_value_proxy = risk_record
                .and_then(|item| item.avoided_loss_value)
                .or_else(|| {
                    risk_record
                        .and_then(|item| item.net_return_pct)
                        .map(f64::abs)
                });
            comparable.reason_codes = stable_reason_codes(
                &comparable
                    .reason_codes
                    .into_iter()
                    .chain(generated.iter().flat_map(|item| item.reason_codes.clone()))
                    .collect::<Vec<_>>(),
            );
            comparable
        })
        .collect()
}

fn rows_from_outcome_coverage_bundle(
    bundle: &CommitteeOutcomeCoverageBundle,
    source_path: &str,
) -> Vec<ComparableCommitteeEvidenceRow> {
    bundle
        .coverage_report
        .cells
        .iter()
        .map(|cell| {
            summary_row_from_coverage_cell(cell, &bundle.coverage_report.coverage_id, source_path)
        })
        .collect()
}

fn rows_from_committee_benchmark_bundle(
    bundle: &CommitteeBenchmarkBundle,
    source_path: &str,
) -> Vec<ComparableCommitteeEvidenceRow> {
    let replay_by_row = bundle
        .replay_report
        .records
        .iter()
        .map(|record| (record.scenario_row.scenario_row_id.clone(), record))
        .collect::<BTreeMap<_, _>>();
    bundle
        .materialized_scenario_set
        .rows
        .iter()
        .map(|row| {
            let mut comparable = ComparableCommitteeEvidenceRow::from_scenario_row(row);
            comparable.row_id =
                format!("committee-benchmark:{source_path}:{}", row.scenario_row_id);
            if let Some(replay) = replay_by_row.get(&row.scenario_row_id) {
                apply_replay_details(&mut comparable, Some(replay), None);
            }
            comparable
        })
        .collect()
}

fn rows_from_source_benchmark_report(
    report: &SourceAwareBenchmarkReport,
    source_path: &str,
) -> Vec<ComparableCommitteeEvidenceRow> {
    let mut rows = Vec::new();
    if let Some(summary) = &report.official_summary {
        rows.push(summary_row_from_source_summary(
            summary,
            &format!("{}:official", report.benchmark_id),
            source_path,
            ComparableEvidenceSourceClass::OfficialNonCrypto,
        ));
    }
    if let Some(summary) = &report.yfinance_summary {
        rows.push(summary_row_from_source_summary(
            summary,
            &format!("{}:yfinance", report.benchmark_id),
            source_path,
            ComparableEvidenceSourceClass::YFinanceResearch,
        ));
    }
    rows
}

fn rows_from_yahoo_report(
    report: &YahooResearchEvidenceReport,
    source_path: &str,
) -> Vec<ComparableCommitteeEvidenceRow> {
    report
        .yfinance_symbols
        .iter()
        .map(|symbol| ComparableCommitteeEvidenceRow {
            row_id: format!("yahoo-report:{}:{}", report.research_id, symbol),
            symbol: symbol.clone(),
            market: infer_market_from_symbol(symbol),
            timeframe: "summary".to_string(),
            horizon_bars: 0,
            timestamp_ms: 0,
            source_kind: format!("YahooResearchEvidenceReport:{source_path}"),
            source_class: ComparableEvidenceSourceClass::YFinanceResearch,
            scenario_row_id: None,
            committee_decision_id: None,
            committee_final_action: "SummaryOnly".to_string(),
            chair_decision: None,
            risk_governor_decision: None,
            baseline_action: None,
            external_action: None,
            no_trade_baseline_action: "NoTrade".to_string(),
            outcome_label: None,
            net_return_pct: None,
            cost_bps: 0.0,
            slippage_bps: 0.0,
            committee_vs_baseline_delta: None,
            committee_vs_notrade_delta: None,
            risk_denied_value_proxy: None,
            no_trade_value_proxy: None,
            outcome_reference_available: false,
            baseline_reference_available: false,
            no_trade_counterfactual_available: false,
            risk_denied_counterfactual_available: false,
            external_reference_available: false,
            row_level: false,
            summary_derived: true,
            no_lookahead_safe: false,
            official_readiness_eligible: false,
            diagnostic_only: true,
            candle_coverage_available: false,
            matched_candle_series_id: None,
            candle_match_status: None,
            candle_official_ready_match: false,
            candle_benchmark_ready_match: false,
            candle_diagnostic_only: false,
            reason_codes: stable_reason_codes(
                &report
                    .reason_codes
                    .iter()
                    .cloned()
                    .chain([ReasonCode::SummaryDerived, ReasonCode::YFinanceResearchOnly])
                    .collect::<Vec<_>>(),
            ),
        })
        .collect()
}

fn rows_from_scorecard(
    scorecard: &CorePerformanceScorecard,
    source_path: &str,
) -> Vec<ComparableCommitteeEvidenceRow> {
    let source_class = if scorecard.artifact_inventory.non_crypto_official_count > 0 {
        ComparableEvidenceSourceClass::OfficialNonCrypto
    } else if scorecard.artifact_inventory.crypto_only_count > 0 {
        ComparableEvidenceSourceClass::OfficialCryptoOnly
    } else if scorecard.artifact_inventory.research_only_count > 0 {
        ComparableEvidenceSourceClass::YFinanceResearch
    } else if scorecard.artifact_inventory.fixture_only_count > 0 {
        ComparableEvidenceSourceClass::FixtureArchitectureTest
    } else {
        ComparableEvidenceSourceClass::ControlledDiagnostic
    };
    vec![ComparableCommitteeEvidenceRow {
        row_id: format!("core-scorecard:{}", scorecard.scorecard_id),
        symbol: "SUMMARY".to_string(),
        market: ProviderMarket::USEquity,
        timeframe: "summary".to_string(),
        horizon_bars: 0,
        timestamp_ms: 0,
        source_kind: format!("CorePerformanceScorecard:{source_path}"),
        source_class,
        scenario_row_id: None,
        committee_decision_id: None,
        committee_final_action: format!("{:?}", scorecard.final_status),
        chair_decision: None,
        risk_governor_decision: Some(format!("{:?}", scorecard.risk_governor_value_report.status)),
        baseline_action: None,
        external_action: None,
        no_trade_baseline_action: "NoTrade".to_string(),
        outcome_label: Some(format!(
            "{:?}",
            scorecard.bottleneck_report.primary_bottleneck
        )),
        net_return_pct: scorecard
            .committee_value_attribution_report
            .committee_vs_baseline_delta,
        cost_bps: 0.0,
        slippage_bps: 0.0,
        committee_vs_baseline_delta: scorecard
            .committee_value_attribution_report
            .committee_vs_baseline_delta,
        committee_vs_notrade_delta: scorecard
            .committee_value_attribution_report
            .committee_vs_no_trade_delta,
        risk_denied_value_proxy: Some(scorecard.risk_governor_value_report.avoided_loss_total),
        no_trade_value_proxy: Some(scorecard.no_trade_value_report.avoided_loss_value),
        outcome_reference_available: scorecard.signal_quality_report.outcome_linked_rows > 0,
        baseline_reference_available: scorecard.signal_quality_report.baseline_reference_rows > 0,
        no_trade_counterfactual_available: scorecard.no_trade_value_report.no_trade_counterfactuals
            > 0,
        risk_denied_counterfactual_available: scorecard
            .risk_governor_value_report
            .risk_denied_counterfactual_count
            > 0,
        external_reference_available: scorecard.signal_quality_report.external_reference_rows > 0,
        row_level: false,
        summary_derived: true,
        no_lookahead_safe: scorecard.signal_quality_report.outcome_linked_rows > 0,
        official_readiness_eligible: false,
        diagnostic_only: matches!(
            scorecard.final_status,
            CorePerformanceFinalStatus::CoreDiagnosticOnly
        ),
        candle_coverage_available: false,
        matched_candle_series_id: None,
        candle_match_status: None,
        candle_official_ready_match: false,
        candle_benchmark_ready_match: false,
        candle_diagnostic_only: false,
        reason_codes: stable_reason_codes(
            &scorecard
                .reason_codes
                .iter()
                .cloned()
                .chain([ReasonCode::SummaryDerived])
                .collect::<Vec<_>>(),
        ),
    }]
}

fn row_from_linked_row(
    linked: &OutcomeLinkedCommitteeScenarioRow,
    replay: Option<&CommitteeReplayRecord>,
    no_trade_record: Option<&CommitteeCounterfactualRecord>,
    risk_record: Option<&CommitteeCounterfactualRecord>,
    prefix: &str,
) -> ComparableCommitteeEvidenceRow {
    let row = &linked.scenario_row;
    let outcome_reference = linked.outcome_reference.as_ref();
    let baseline_reference = linked.baseline_reference.as_ref();
    let external_reference = linked.external_reference.as_ref();
    let mut comparable = ComparableCommitteeEvidenceRow::from_scenario_row(row);
    comparable.row_id = format!("{prefix}:{}", row.scenario_row_id);
    comparable.committee_decision_id =
        replay.map(|record| record.chair_decision_record.decision_id.clone());
    comparable.baseline_action = baseline_reference
        .map(|item| item.baseline_action.as_summary_str().to_string())
        .or_else(|| comparable.baseline_action.clone());
    comparable.external_action = external_reference
        .and_then(|item| item.external_action.clone())
        .or_else(|| comparable.external_action.clone());
    comparable.outcome_label =
        outcome_reference.map(|item| format!("{:?}", item.triple_barrier_label));
    comparable.net_return_pct = outcome_reference.and_then(|item| item.net_return_pct);
    comparable.cost_bps = outcome_reference
        .map(|item| item.cost_bps)
        .unwrap_or_default();
    comparable.slippage_bps = outcome_reference
        .map(|item| item.slippage_bps)
        .unwrap_or_default();
    comparable.outcome_reference_available =
        outcome_reference.is_some() || comparable.outcome_reference_available;
    comparable.baseline_reference_available =
        baseline_reference.is_some() || comparable.baseline_reference_available;
    comparable.external_reference_available =
        external_reference.is_some() || comparable.external_reference_available;
    comparable.no_trade_counterfactual_available =
        no_trade_record.is_some() || comparable.no_trade_counterfactual_available;
    comparable.risk_denied_counterfactual_available =
        risk_record.is_some() || comparable.risk_denied_counterfactual_available;
    comparable.no_lookahead_safe = outcome_reference
        .map(|item| item.no_lookahead_safe)
        .or_else(|| no_trade_record.map(|item| item.no_lookahead_safe))
        .or_else(|| risk_record.map(|item| item.no_lookahead_safe))
        .unwrap_or(comparable.no_lookahead_safe);
    comparable.diagnostic_only = comparable.diagnostic_only
        || no_trade_record.is_some_and(|item| item.diagnostic_only)
        || risk_record.is_some_and(|item| item.diagnostic_only);
    comparable.official_readiness_eligible = comparable.official_readiness_eligible
        && outcome_reference.is_some()
        && !comparable.diagnostic_only;
    comparable.no_trade_value_proxy = no_trade_record
        .and_then(|item| item.avoided_loss_value)
        .or_else(|| {
            no_trade_record
                .and_then(|item| item.net_return_pct)
                .map(f64::abs)
        });
    comparable.risk_denied_value_proxy = risk_record
        .and_then(|item| item.avoided_loss_value)
        .or_else(|| {
            risk_record
                .and_then(|item| item.net_return_pct)
                .map(f64::abs)
        });
    apply_replay_details(&mut comparable, replay, outcome_reference);
    comparable.reason_codes = stable_reason_codes(
        &comparable
            .reason_codes
            .into_iter()
            .chain(linked.reason_codes.clone())
            .chain(
                outcome_reference
                    .into_iter()
                    .flat_map(|item| item.reason_codes.clone()),
            )
            .chain(
                baseline_reference
                    .into_iter()
                    .flat_map(|item| item.reason_codes.clone()),
            )
            .chain(
                external_reference
                    .into_iter()
                    .flat_map(|item| item.reason_codes.clone()),
            )
            .chain(
                no_trade_record
                    .into_iter()
                    .flat_map(|item| item.reason_codes.clone()),
            )
            .chain(
                risk_record
                    .into_iter()
                    .flat_map(|item| item.reason_codes.clone()),
            )
            .collect::<Vec<_>>(),
    );
    comparable
}

fn apply_replay_details(
    comparable: &mut ComparableCommitteeEvidenceRow,
    replay: Option<&CommitteeReplayRecord>,
    outcome_reference: Option<&super::committee_outcome_reference::CommitteeOutcomeReference>,
) {
    let Some(replay) = replay else {
        return;
    };
    comparable.committee_decision_id = Some(replay.chair_decision_record.decision_id.clone());
    comparable.committee_final_action = format!("{:?}", replay.final_action);
    comparable.chair_decision = Some(format!("{:?}", replay.chair_decision_record.final_decision));
    comparable.risk_governor_decision = Some(format!(
        "{:?}",
        replay.risk_bridge_outcome.risk_decision.kind
    ));
    if let Some(outcome) = outcome_reference {
        let committee_return = committee_return_proxy(replay.final_action, outcome);
        comparable.committee_vs_notrade_delta = Some(committee_return);
        if let Some(action) = &comparable.baseline_action {
            let baseline_action = CommitteeBaselineAction::from_summary(action);
            comparable.committee_vs_baseline_delta = Some(
                committee_return - scaled_return(baseline_action.sizing_multiplier(), outcome),
            );
        }
        if replay.final_action == CommitteeFinalAction::FinalDenied {
            comparable.risk_denied_value_proxy = comparable.risk_denied_value_proxy.or_else(|| {
                outcome
                    .cost_adjusted_return_pct()
                    .map(|value| value.min(0.0).abs())
            });
        }
        if replay.final_action == CommitteeFinalAction::FinalNoTrade {
            comparable.no_trade_value_proxy = comparable.no_trade_value_proxy.or_else(|| {
                outcome
                    .cost_adjusted_return_pct()
                    .map(|value| value.min(0.0).abs())
            });
        }
    }
    comparable.reason_codes = stable_reason_codes(
        &comparable
            .reason_codes
            .clone()
            .into_iter()
            .chain(replay.reason_codes.clone())
            .chain(replay.chair_decision_record.reason_codes.clone())
            .chain(replay.risk_bridge_outcome.reason_codes.clone())
            .collect::<Vec<_>>(),
    );
}

fn summary_row_from_coverage_cell(
    cell: &OutcomeCoverageCell,
    coverage_id: &str,
    source_path: &str,
) -> ComparableCommitteeEvidenceRow {
    let source_class = source_class_from_summary(&cell.source_kind, cell.market);
    let diagnostic_only = matches!(
        source_class,
        ComparableEvidenceSourceClass::ControlledDiagnostic
            | ComparableEvidenceSourceClass::YFinanceResearch
            | ComparableEvidenceSourceClass::FixtureArchitectureTest
            | ComparableEvidenceSourceClass::SyntheticTest
    );
    ComparableCommitteeEvidenceRow {
        row_id: format!(
            "coverage:{}:{}:{}:{}:{}",
            coverage_id, cell.source_kind, cell.symbol, cell.timeframe, cell.horizon_bars
        ),
        symbol: cell.symbol.clone(),
        market: cell.market,
        timeframe: cell.timeframe.clone(),
        horizon_bars: cell.horizon_bars,
        timestamp_ms: 0,
        source_kind: format!("CommitteeOutcomeCoverage:{source_path}"),
        source_class,
        scenario_row_id: None,
        committee_decision_id: None,
        committee_final_action: "SummaryOnly".to_string(),
        chair_decision: None,
        risk_governor_decision: None,
        baseline_action: None,
        external_action: None,
        no_trade_baseline_action: "NoTrade".to_string(),
        outcome_label: Some(format!("rows={}", cell.row_count)),
        net_return_pct: None,
        cost_bps: 0.0,
        slippage_bps: 0.0,
        committee_vs_baseline_delta: None,
        committee_vs_notrade_delta: None,
        risk_denied_value_proxy: None,
        no_trade_value_proxy: None,
        outcome_reference_available: cell.outcome_linked_count > 0,
        baseline_reference_available: cell.baseline_linked_count > 0,
        no_trade_counterfactual_available: cell.no_trade_counterfactual_count > 0,
        risk_denied_counterfactual_available: cell.risk_denied_counterfactual_count > 0,
        external_reference_available: cell.external_linked_count > 0,
        row_level: false,
        summary_derived: true,
        no_lookahead_safe: cell.no_lookahead_safe_count >= cell.outcome_linked_count.max(1),
        official_readiness_eligible: false,
        diagnostic_only,
        candle_coverage_available: false,
        matched_candle_series_id: None,
        candle_match_status: None,
        candle_official_ready_match: false,
        candle_benchmark_ready_match: false,
        candle_diagnostic_only: false,
        reason_codes: stable_reason_codes(
            &cell
                .reason_codes
                .iter()
                .cloned()
                .chain([ReasonCode::SummaryDerived])
                .collect::<Vec<_>>(),
        ),
    }
}

fn summary_row_from_source_summary(
    summary: &crate::experiment::SourceBenchmarkSummary,
    summary_id: &str,
    source_path: &str,
    source_class: ComparableEvidenceSourceClass,
) -> ComparableCommitteeEvidenceRow {
    ComparableCommitteeEvidenceRow {
        row_id: format!("source-benchmark:{summary_id}"),
        symbol: summary.source_label.clone(),
        market: infer_market_from_symbol(&summary.source_label),
        timeframe: "summary".to_string(),
        horizon_bars: 0,
        timestamp_ms: 0,
        source_kind: format!("SourceAwareBenchmark:{source_path}"),
        source_class,
        scenario_row_id: None,
        committee_decision_id: None,
        committee_final_action: "SummaryOnly".to_string(),
        chair_decision: None,
        risk_governor_decision: None,
        baseline_action: None,
        external_action: None,
        no_trade_baseline_action: "NoTrade".to_string(),
        outcome_label: summary.status_label.clone(),
        net_return_pct: summary.avg_net_return_pct,
        cost_bps: 0.0,
        slippage_bps: 0.0,
        committee_vs_baseline_delta: None,
        committee_vs_notrade_delta: None,
        risk_denied_value_proxy: summary.defensive_value,
        no_trade_value_proxy: summary.opportunity_cost,
        outcome_reference_available: summary.total_outcome_records > 0,
        baseline_reference_available: false,
        no_trade_counterfactual_available: false,
        risk_denied_counterfactual_available: false,
        external_reference_available: false,
        row_level: false,
        summary_derived: true,
        no_lookahead_safe: summary.total_outcome_records > 0,
        official_readiness_eligible: false,
        diagnostic_only: source_class != ComparableEvidenceSourceClass::OfficialNonCrypto,
        candle_coverage_available: false,
        matched_candle_series_id: None,
        candle_match_status: None,
        candle_official_ready_match: false,
        candle_benchmark_ready_match: false,
        candle_diagnostic_only: false,
        reason_codes: stable_reason_codes(
            &summary
                .reason_codes
                .iter()
                .cloned()
                .chain([ReasonCode::SummaryDerived])
                .collect::<Vec<_>>(),
        ),
    }
}

fn source_class_from_summary(
    source_kind: &str,
    market: ProviderMarket,
) -> ComparableEvidenceSourceClass {
    let lower = source_kind.to_ascii_lowercase();
    if lower.contains("yfinance") || lower.contains("research") {
        ComparableEvidenceSourceClass::YFinanceResearch
    } else if lower.contains("fixture") {
        ComparableEvidenceSourceClass::FixtureArchitectureTest
    } else if lower.contains("synthetic") {
        ComparableEvidenceSourceClass::SyntheticTest
    } else if lower.contains("reallocal") || lower.contains("controlled") {
        ComparableEvidenceSourceClass::ControlledDiagnostic
    } else if lower.contains("official") && market == ProviderMarket::Crypto {
        ComparableEvidenceSourceClass::OfficialCryptoOnly
    } else if lower.contains("official") {
        ComparableEvidenceSourceClass::OfficialNonCrypto
    } else {
        ComparableEvidenceSourceClass::Unknown
    }
}

fn dedupe_rows(
    config: &ComparableCommitteeEvidenceConfig,
    rows: Vec<ComparableCommitteeEvidenceRow>,
) -> Vec<ComparableCommitteeEvidenceRow> {
    let mut selected = BTreeMap::<String, ComparableCommitteeEvidenceRow>::new();
    for row in rows {
        let key = canonical_row_key(&row);
        match selected.get(&key) {
            None => {
                selected.insert(key, row);
            }
            Some(existing) if prefer_row(&row, existing, config) => {
                selected.insert(key, row);
            }
            Some(_) => {}
        }
    }
    selected.into_values().collect()
}

fn bound_rows(
    config: &ComparableCommitteeEvidenceConfig,
    rows: Vec<ComparableCommitteeEvidenceRow>,
) -> Vec<ComparableCommitteeEvidenceRow> {
    let mut sorted = rows;
    sorted.sort_by(|left, right| {
        left.row_id
            .cmp(&right.row_id)
            .then(left.symbol.cmp(&right.symbol))
            .then(left.timestamp_ms.cmp(&right.timestamp_ms))
    });
    let allowed_symbols = sorted
        .iter()
        .map(|row| row.symbol.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .take(config.max_symbols)
        .collect::<BTreeSet<_>>();
    let mut bounded = sorted
        .into_iter()
        .filter(|row| allowed_symbols.contains(&row.symbol))
        .take(config.max_rows)
        .collect::<Vec<_>>();
    while serde_json::to_vec(&bounded)
        .map(|bytes| bytes.len() > config.max_bytes)
        .unwrap_or(false)
    {
        bounded.pop();
    }
    bounded
}

fn prefer_row(
    candidate: &ComparableCommitteeEvidenceRow,
    current: &ComparableCommitteeEvidenceRow,
    config: &ComparableCommitteeEvidenceConfig,
) -> bool {
    candidate.completeness_score(config) > current.completeness_score(config)
        || (candidate.completeness_score(config) == current.completeness_score(config)
            && candidate.row_id < current.row_id)
}

fn canonical_row_key(row: &ComparableCommitteeEvidenceRow) -> String {
    row.scenario_row_id.clone().unwrap_or_else(|| {
        format!(
            "{}:{}:{}:{}:{:?}",
            row.symbol, row.timestamp_ms, row.timeframe, row.horizon_bars, row.source_class
        )
    })
}

fn committee_return_proxy(
    final_action: CommitteeFinalAction,
    outcome: &super::committee_outcome_reference::CommitteeOutcomeReference,
) -> f64 {
    match final_action {
        CommitteeFinalAction::PaperApprove => scaled_return(1.0, outcome),
        CommitteeFinalAction::PaperReduceSize | CommitteeFinalAction::HumanConfirmRequired => {
            scaled_return(0.5, outcome)
        }
        CommitteeFinalAction::FinalNoTrade | CommitteeFinalAction::FinalDenied => 0.0,
    }
}

fn scaled_return(
    multiplier: f64,
    outcome: &super::committee_outcome_reference::CommitteeOutcomeReference,
) -> f64 {
    multiplier * outcome.cost_adjusted_return_pct().unwrap_or_default()
}

fn parse_json_file<T: DeserializeOwned>(path: &str) -> Result<Option<T>, String> {
    let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
    Ok(serde_json::from_str(&text).ok())
}

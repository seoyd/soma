use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use serde_json::Value;

use crate::CandleSeries;
use crate::core::{ReasonCode, stable_reason_codes};
use crate::data::EvidenceSourceKind;

use super::baseline_reference_generator::{
    BaselineReferenceGenerator, BaselineReferencePolicy, BaselineReferenceSource,
    LoadedBaselineReference,
};
use super::candle_alignment::{
    CandleAligner, CandleAlignmentRecord, CandleAlignmentReport, CandleAlignmentStatus,
};
use super::committee_counterfactual_builder::{load_local_candle_series_map, normalize_symbol};
use super::committee_reference_pack::{
    CommitteeReferencePackConfig, GeneratedCommitteeReference, GeneratedCommitteeReferencePack,
    GeneratedReferenceKind, GeneratedReferenceSource, GeneratedReferenceStatus,
};
use super::committee_reference_pack_bundle::CommitteeReferencePackBundle;
use super::committee_scenario_loader::{
    CommitteeScenarioLoadConfig, CommitteeScenarioLoader, CommitteeScenarioRow,
    CommitteeScenarioSet, CommitteeScenarioSourceKind,
};
use super::counterfactual_reference_generator::{
    CounterfactualReferenceGenerator, CounterfactualReferencePolicy,
};
use super::official_committee_pack::{
    OfficialCommitteeScenarioPack, OfficialCommitteeScenarioPackBuilder,
    OfficialCommitteeScenarioPackConfig,
};
use super::reference_pack_quality::build_reference_pack_quality_report;
use super::sufficiency_closure::{SufficiencyClosureConfig, SufficiencyClosureRunner};
use super::triple_barrier_reference_builder::{
    TripleBarrierReferenceBuilder, TripleBarrierReferenceConfig,
};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CommitteeReferencePackRunner;

#[derive(Clone, Debug, PartialEq)]
struct LoadedExternalReference {
    symbol: String,
    timestamp_ms: u64,
    horizon_bars: Option<usize>,
    reference: super::committee_outcome_reference::CommitteeExternalReference,
}

impl CommitteeReferencePackRunner {
    pub fn align_candles(
        &self,
        config: &CommitteeReferencePackConfig,
    ) -> Result<CandleAlignmentReport, String> {
        config.validate()?;
        let scenario_rows = load_scenario_rows(config)?;
        let candle_series = load_local_candle_series_map(&config.candle_series_paths)?;
        Ok(CandleAligner::default().align_rows(&scenario_rows, &candle_series, config))
    }

    pub fn build_reference_pack(
        &self,
        config: &CommitteeReferencePackConfig,
    ) -> Result<GeneratedCommitteeReferencePack, String> {
        config.validate()?;
        let scenario_rows = load_scenario_rows(config)?;
        validate_scope(config, &scenario_rows)?;
        let candle_series = load_local_candle_series_map(&config.candle_series_paths)?;
        let alignment_report =
            CandleAligner::default().align_rows(&scenario_rows, &candle_series, config);
        let baseline_artifacts =
            BaselineReferenceGenerator::load_existing(&config.baseline_reference_paths)?;
        let external_artifacts = load_external_artifacts(&config.external_prediction_paths)?;
        let outcome_config = TripleBarrierReferenceConfig {
            horizon_bars: config.default_horizon_bars,
            take_profit_pct: config.default_take_profit_pct,
            stop_loss_pct: config.default_stop_loss_pct,
            cost_bps: config.default_cost_bps,
            slippage_bps: config.default_slippage_bps,
            tie_break_policy:
                super::triple_barrier_reference_builder::TripleBarrierTieBreakPolicy::StopFirst,
            reason_codes: vec![ReasonCode::CommitteeReferencePackBuilt],
        };
        let counterfactual_policy = CounterfactualReferencePolicy::default();
        let mut generated_references = Vec::new();
        for row in &scenario_rows {
            let alignment = alignment_report
                .records
                .iter()
                .find(|record| record.scenario_row_id == row.scenario_row_id)
                .cloned()
                .ok_or_else(|| format!("missing alignment record for {}", row.scenario_row_id))?;
            let series = candle_series.get(&normalize_symbol(&row.symbol));
            let row_allowed = row_allowed_by_source(row, config);
            if config.build_triple_barrier_outcomes {
                generated_references.push(self.generate_outcome_reference(
                    row,
                    &alignment,
                    series,
                    &outcome_config,
                    row_allowed,
                    config.allow_estimated_references,
                ));
            }
            if config.build_baseline_references {
                generated_references.push(self.generate_baseline_reference(
                    row,
                    &baseline_artifacts,
                    config.default_horizon_bars,
                    config.allow_estimated_references,
                ));
            }
            if let Some(reference) = self.generate_external_reference(
                row,
                &external_artifacts,
                config.default_horizon_bars,
                row_allowed,
            ) {
                generated_references.push(reference);
            }
            if config.build_no_trade_counterfactuals {
                generated_references.push(self.generate_counterfactual_reference(
                    row,
                    &alignment,
                    series,
                    &outcome_config,
                    &counterfactual_policy,
                    GeneratedReferenceKind::NoTradeCounterfactual,
                    row_allowed,
                    config.allow_estimated_references,
                ));
            }
            if config.build_risk_denied_counterfactuals {
                generated_references.push(self.generate_counterfactual_reference(
                    row,
                    &alignment,
                    series,
                    &outcome_config,
                    &counterfactual_policy,
                    GeneratedReferenceKind::RiskDeniedCounterfactual,
                    row_allowed,
                    config.allow_estimated_references,
                ));
            }
        }
        Ok(GeneratedCommitteeReferencePack::new(
            config.reference_pack_id.clone(),
            scenario_rows,
            generated_references,
            alignment_report,
            stable_reason_codes(&[
                ReasonCode::CommitteeReferencePackBuilt,
                ReasonCode::CommitteeReferencePackRunnerBuilt,
            ]),
        ))
    }

    pub fn run(
        &self,
        config: &CommitteeReferencePackConfig,
    ) -> Result<CommitteeReferencePackBundle, String> {
        let pack = self.build_reference_pack(config)?;
        let quality_report = build_reference_pack_quality_report(config, &pack);
        let closure_config = SufficiencyClosureConfig {
            closure_id: format!("{}-closure", config.reference_pack_id),
            generated_reference_pack_path: config
                .output_dir()
                .join("generated_reference_pack.json")
                .display()
                .to_string(),
            output_root: config.output_root.clone(),
            ..SufficiencyClosureConfig::default()
        };
        let closure_report =
            Some(SufficiencyClosureRunner::default().run_with_pack(&closure_config, &pack)?);
        let bundle = CommitteeReferencePackBundle::new(
            pack,
            quality_report,
            closure_report,
            format!(
                "output_dir={};input_paths={}",
                config.output_dir().display(),
                config
                    .scenario_pack_paths
                    .iter()
                    .chain(config.scenario_set_paths.iter())
                    .chain(config.candle_series_paths.iter())
                    .cloned()
                    .collect::<Vec<_>>()
                    .join("|")
            ),
        );
        bundle.write_to_dir(&config.output_dir())?;
        Ok(bundle)
    }

    fn generate_outcome_reference(
        &self,
        row: &CommitteeScenarioRow,
        alignment: &CandleAlignmentRecord,
        series: Option<&CandleSeries>,
        outcome_config: &TripleBarrierReferenceConfig,
        row_allowed: bool,
        allow_estimated_references: bool,
    ) -> GeneratedCommitteeReference {
        let reference_id = format!("{}-outcome", row.scenario_row_id);
        if !row_allowed {
            return skipped_reference(
                reference_id,
                row,
                GeneratedReferenceKind::TripleBarrierOutcome,
                GeneratedReferenceStatus::SkippedSourceNotAllowed,
                GeneratedReferenceSource::Unknown,
                false,
                vec![ReasonCode::ReadinessEvidenceExcluded],
            );
        }
        let Some(series) = series else {
            return skipped_reference(
                reference_id,
                row,
                GeneratedReferenceKind::TripleBarrierOutcome,
                GeneratedReferenceStatus::SkippedNoCandleMatch,
                GeneratedReferenceSource::Unknown,
                false,
                vec![ReasonCode::MissingRealLocalData],
            );
        };
        if matches!(alignment.status, CandleAlignmentStatus::RejectedNoLookahead) {
            return skipped_reference(
                reference_id,
                row,
                GeneratedReferenceKind::TripleBarrierOutcome,
                GeneratedReferenceStatus::RejectedNoLookahead,
                GeneratedReferenceSource::LocalCandleSeries,
                false,
                vec![ReasonCode::RejectedNoLookaheadReference],
            );
        }
        if matches!(
            alignment.status,
            CandleAlignmentStatus::GapDetected
                | CandleAlignmentStatus::DuplicateTimestamp
                | CandleAlignmentStatus::BadDataQuality
        ) {
            return skipped_reference(
                reference_id,
                row,
                GeneratedReferenceKind::TripleBarrierOutcome,
                GeneratedReferenceStatus::RejectedBadDataQuality,
                GeneratedReferenceSource::LocalCandleSeries,
                false,
                vec![ReasonCode::DataQualityTooLow],
            );
        }
        if matches!(
            alignment.status,
            CandleAlignmentStatus::WrongHorizon | CandleAlignmentStatus::InsufficientFutureBars
        ) {
            return skipped_reference(
                reference_id,
                row,
                GeneratedReferenceKind::TripleBarrierOutcome,
                GeneratedReferenceStatus::SkippedNoOutcomeWindow,
                GeneratedReferenceSource::LocalCandleSeries,
                false,
                vec![ReasonCode::InsufficientBars],
            );
        }
        if !matches!(
            alignment.status,
            CandleAlignmentStatus::MatchedExact | CandleAlignmentStatus::MatchedWithTolerance
        ) {
            return skipped_reference(
                reference_id,
                row,
                GeneratedReferenceKind::TripleBarrierOutcome,
                GeneratedReferenceStatus::SkippedNoCandleMatch,
                GeneratedReferenceSource::Unknown,
                false,
                vec![ReasonCode::MissingRealLocalData],
            );
        }
        let diagnostic_only = alignment.status == CandleAlignmentStatus::MatchedWithTolerance
            && allow_estimated_references;
        match TripleBarrierReferenceBuilder::default().build(
            row,
            alignment,
            series,
            outcome_config,
            diagnostic_only,
        ) {
            Ok(result) => GeneratedCommitteeReference {
                reference_id,
                scenario_row_id: row.scenario_row_id.clone(),
                reference_kind: GeneratedReferenceKind::TripleBarrierOutcome,
                status: if result.diagnostic_only {
                    GeneratedReferenceStatus::DiagnosticOnlyEstimated
                } else {
                    GeneratedReferenceStatus::Generated
                },
                outcome_reference: Some(result.reference.clone()),
                baseline_reference: None,
                external_reference: None,
                no_trade_counterfactual: None,
                risk_denied_counterfactual: None,
                generated_from: GeneratedReferenceSource::from(result.generated_from),
                official_readiness_eligible: official_readiness_eligible(
                    row,
                    !result.diagnostic_only,
                    true,
                    result.reference.no_lookahead_safe,
                ),
                diagnostic_only: result.diagnostic_only,
                reason_codes: stable_reason_codes(&result.reference.reason_codes),
            },
            Err(err) => skipped_reference(
                reference_id,
                row,
                GeneratedReferenceKind::TripleBarrierOutcome,
                GeneratedReferenceStatus::SkippedNoOutcomeWindow,
                GeneratedReferenceSource::LocalCandleSeries,
                false,
                vec![
                    ReasonCode::DataLoadFailed,
                    ReasonCode::CommitteeOutcomeReferenceBuilt,
                    if err.contains("future window") {
                        ReasonCode::InsufficientBars
                    } else {
                        ReasonCode::DataValidationFailed
                    },
                ],
            ),
        }
    }

    fn generate_baseline_reference(
        &self,
        row: &CommitteeScenarioRow,
        baseline_artifacts: &[LoadedBaselineReference],
        default_horizon_bars: usize,
        allow_estimated_references: bool,
    ) -> GeneratedCommitteeReference {
        let reference_id = format!("{}-baseline", row.scenario_row_id);
        let existing = BaselineReferenceGenerator::find_existing(
            row,
            baseline_artifacts,
            default_horizon_bars,
        );
        let policy = if existing.is_some() {
            BaselineReferencePolicy {
                source: BaselineReferenceSource::ExistingArtifact,
                ..BaselineReferencePolicy::default()
            }
        } else if row.baseline_signal_summary.is_some() {
            BaselineReferencePolicy {
                source: BaselineReferenceSource::DeterministicBaselineSignalApprox,
                allow_approximation: allow_estimated_references,
                ..BaselineReferencePolicy::default()
            }
        } else {
            BaselineReferencePolicy::default()
        };
        let generated = BaselineReferenceGenerator::default().generate(row, existing, &policy);
        GeneratedCommitteeReference {
            reference_id,
            scenario_row_id: row.scenario_row_id.clone(),
            reference_kind: GeneratedReferenceKind::BaselineAction,
            status: if generated.diagnostic_only {
                GeneratedReferenceStatus::DiagnosticOnlyEstimated
            } else {
                GeneratedReferenceStatus::Generated
            },
            outcome_reference: None,
            baseline_reference: Some(generated.reference),
            external_reference: None,
            no_trade_counterfactual: None,
            risk_denied_counterfactual: None,
            generated_from: GeneratedReferenceSource::from(generated.source),
            official_readiness_eligible: existing.is_some()
                && official_readiness_eligible(row, true, false, true),
            diagnostic_only: generated.diagnostic_only,
            reason_codes: generated.reason_codes,
        }
    }

    fn generate_external_reference(
        &self,
        row: &CommitteeScenarioRow,
        external_artifacts: &[LoadedExternalReference],
        default_horizon_bars: usize,
        row_allowed: bool,
    ) -> Option<GeneratedCommitteeReference> {
        let reference_id = format!("{}-external", row.scenario_row_id);
        if !row_allowed {
            return Some(skipped_reference(
                reference_id,
                row,
                GeneratedReferenceKind::ExternalPredictionAction,
                GeneratedReferenceStatus::SkippedSourceNotAllowed,
                GeneratedReferenceSource::Unknown,
                false,
                vec![ReasonCode::ReadinessEvidenceExcluded],
            ));
        }
        let horizon_bars = super::committee_counterfactual_builder::horizon_bars_for_row(
            row,
            default_horizon_bars,
        );
        let found = external_artifacts.iter().find(|item| {
            item.symbol == normalize_symbol(&row.symbol)
                && item.timestamp_ms == row.timestamp_ms
                && item.horizon_bars.unwrap_or(horizon_bars) == horizon_bars
        })?;
        Some(GeneratedCommitteeReference {
            reference_id,
            scenario_row_id: row.scenario_row_id.clone(),
            reference_kind: GeneratedReferenceKind::ExternalPredictionAction,
            status: if found.reference.prediction_schema_valid {
                GeneratedReferenceStatus::Generated
            } else {
                GeneratedReferenceStatus::SkippedInvalidPrediction
            },
            outcome_reference: None,
            baseline_reference: None,
            external_reference: Some(found.reference.clone()),
            no_trade_counterfactual: None,
            risk_denied_counterfactual: None,
            generated_from: GeneratedReferenceSource::ExistingArtifact,
            official_readiness_eligible: false,
            diagnostic_only: false,
            reason_codes: stable_reason_codes(&found.reference.reason_codes),
        })
    }

    fn generate_counterfactual_reference(
        &self,
        row: &CommitteeScenarioRow,
        alignment: &CandleAlignmentRecord,
        series: Option<&CandleSeries>,
        outcome_config: &TripleBarrierReferenceConfig,
        policy: &CounterfactualReferencePolicy,
        kind: GeneratedReferenceKind,
        row_allowed: bool,
        allow_estimated_references: bool,
    ) -> GeneratedCommitteeReference {
        let reference_id = format!(
            "{}-{}",
            row.scenario_row_id,
            match kind {
                GeneratedReferenceKind::NoTradeCounterfactual => "no-trade",
                GeneratedReferenceKind::RiskDeniedCounterfactual => "risk-denied",
                _ => "counterfactual",
            }
        );
        if !row_allowed {
            return skipped_reference(
                reference_id,
                row,
                kind,
                GeneratedReferenceStatus::SkippedSourceNotAllowed,
                GeneratedReferenceSource::Unknown,
                false,
                vec![ReasonCode::ReadinessEvidenceExcluded],
            );
        }
        let Some(series) = series else {
            return skipped_reference(
                reference_id,
                row,
                kind,
                GeneratedReferenceStatus::SkippedNoCandleMatch,
                GeneratedReferenceSource::Unknown,
                false,
                vec![ReasonCode::MissingRealLocalData],
            );
        };
        let diagnostic_only = alignment.status == CandleAlignmentStatus::MatchedWithTolerance
            && allow_estimated_references
            || !row.evidence_source_kind.readiness_eligible();
        let record = match kind {
            GeneratedReferenceKind::NoTradeCounterfactual => {
                CounterfactualReferenceGenerator::default().generate_no_trade(
                    row,
                    alignment,
                    series,
                    outcome_config,
                    policy,
                    diagnostic_only,
                )
            }
            GeneratedReferenceKind::RiskDeniedCounterfactual => {
                CounterfactualReferenceGenerator::default().generate_risk_denied(
                    row,
                    alignment,
                    series,
                    outcome_config,
                    policy,
                    diagnostic_only,
                )
            }
            _ => unreachable!(),
        };
        let status = match record.build_status {
            super::committee_counterfactual_builder::CounterfactualBuildStatus::Built => GeneratedReferenceStatus::Generated,
            super::committee_counterfactual_builder::CounterfactualBuildStatus::EstimatedDiagnosticOnly => GeneratedReferenceStatus::DiagnosticOnlyEstimated,
            super::committee_counterfactual_builder::CounterfactualBuildStatus::RejectedNoLookahead => GeneratedReferenceStatus::RejectedNoLookahead,
            super::committee_counterfactual_builder::CounterfactualBuildStatus::RejectedBadDataQuality => GeneratedReferenceStatus::RejectedBadDataQuality,
            super::committee_counterfactual_builder::CounterfactualBuildStatus::UnavailableWrongHorizon => GeneratedReferenceStatus::SkippedNoOutcomeWindow,
            super::committee_counterfactual_builder::CounterfactualBuildStatus::UnavailableNoCandleData
            | super::committee_counterfactual_builder::CounterfactualBuildStatus::UnavailableNoTimestampMatch => GeneratedReferenceStatus::SkippedNoCandleMatch,
        };
        GeneratedCommitteeReference {
            reference_id,
            scenario_row_id: row.scenario_row_id.clone(),
            reference_kind: kind,
            status,
            outcome_reference: None,
            baseline_reference: None,
            external_reference: None,
            no_trade_counterfactual: if kind == GeneratedReferenceKind::NoTradeCounterfactual {
                Some(record.clone())
            } else {
                None
            },
            risk_denied_counterfactual: if kind == GeneratedReferenceKind::RiskDeniedCounterfactual
            {
                Some(record.clone())
            } else {
                None
            },
            generated_from: if diagnostic_only {
                GeneratedReferenceSource::EstimatedDiagnosticOnly
            } else {
                GeneratedReferenceSource::LocalCandleSeries
            },
            official_readiness_eligible: official_readiness_eligible(
                row,
                !diagnostic_only,
                true,
                record.no_lookahead_safe,
            ),
            diagnostic_only,
            reason_codes: stable_reason_codes(&record.reason_codes),
        }
    }
}

fn load_scenario_rows(
    config: &CommitteeReferencePackConfig,
) -> Result<Vec<CommitteeScenarioRow>, String> {
    let mut rows = Vec::new();
    for path in &config.scenario_pack_paths {
        let loaded = if path.ends_with(".toml") {
            OfficialCommitteeScenarioPackBuilder::default().build(
                &OfficialCommitteeScenarioPackConfig::from_toml_path(Path::new(path))?,
            )?
        } else {
            OfficialCommitteeScenarioPack::from_json_path(Path::new(path))?
        };
        rows.extend(loaded.rows);
    }
    for path in &config.scenario_set_paths {
        let loaded = if path.ends_with(".toml") {
            CommitteeScenarioLoader::default().load(
                &CommitteeScenarioLoadConfig::from_toml_path(Path::new(path))?,
            )?
        } else {
            CommitteeScenarioSet::from_json_path(Path::new(path))?
        };
        rows.extend(loaded.rows);
    }
    rows.sort_by(|left, right| left.scenario_row_id.cmp(&right.scenario_row_id));
    rows.dedup_by(|left, right| left.scenario_row_id == right.scenario_row_id);
    Ok(rows)
}

fn validate_scope(
    config: &CommitteeReferencePackConfig,
    scenario_rows: &[CommitteeScenarioRow],
) -> Result<(), String> {
    if scenario_rows.len() > config.max_rows {
        return Err(format!(
            "committee reference pack loaded {} rows which exceeds max_rows {}",
            scenario_rows.len(),
            config.max_rows
        ));
    }
    let unique_symbols = scenario_rows
        .iter()
        .map(|row| normalize_symbol(&row.symbol))
        .collect::<BTreeSet<_>>();
    if unique_symbols.len() > config.max_symbols {
        return Err(format!(
            "committee reference pack loaded {} symbols which exceeds max_symbols {}",
            unique_symbols.len(),
            config.max_symbols
        ));
    }
    let storage_bytes = config
        .scenario_pack_paths
        .iter()
        .chain(config.scenario_set_paths.iter())
        .chain(config.candle_series_paths.iter())
        .chain(config.baseline_reference_paths.iter())
        .chain(config.external_prediction_paths.iter())
        .filter_map(|path| fs::metadata(path).ok())
        .map(|metadata| metadata.len() as usize)
        .sum::<usize>();
    if storage_bytes > config.max_bytes {
        return Err(format!(
            "committee reference pack input size {} exceeds max_bytes {}",
            storage_bytes, config.max_bytes
        ));
    }
    Ok(())
}

fn row_allowed_by_source(
    row: &CommitteeScenarioRow,
    config: &CommitteeReferencePackConfig,
) -> bool {
    match row.evidence_source_kind {
        EvidenceSourceKind::YFinanceResearch => config.allow_yfinance_research,
        EvidenceSourceKind::SyntheticFixture | EvidenceSourceKind::TestFixture => {
            config.allow_fixture || config.allow_controlled_fixture_references
        }
        _ => true,
    }
}

fn official_readiness_eligible(
    row: &CommitteeScenarioRow,
    source_allowed: bool,
    local_candle_based: bool,
    no_lookahead_safe: bool,
) -> bool {
    let provenance = row.provenance_summary.to_ascii_lowercase();
    source_allowed
        && local_candle_based
        && no_lookahead_safe
        && row.evidence_source_kind == EvidenceSourceKind::OfficialApiCollected
        && !provenance.contains("missing")
        && (row.source_kind == CommitteeScenarioSourceKind::EvidenceLaneReport
            || provenance.contains("row-level-provenance"))
        && !matches!(
            row.source_kind,
            CommitteeScenarioSourceKind::Fixture | CommitteeScenarioSourceKind::SyntheticTest
        )
}

fn skipped_reference(
    reference_id: String,
    row: &CommitteeScenarioRow,
    reference_kind: GeneratedReferenceKind,
    status: GeneratedReferenceStatus,
    generated_from: GeneratedReferenceSource,
    diagnostic_only: bool,
    reason_codes: Vec<ReasonCode>,
) -> GeneratedCommitteeReference {
    GeneratedCommitteeReference {
        reference_id,
        scenario_row_id: row.scenario_row_id.clone(),
        reference_kind,
        status,
        outcome_reference: None,
        baseline_reference: None,
        external_reference: None,
        no_trade_counterfactual: None,
        risk_denied_counterfactual: None,
        generated_from,
        official_readiness_eligible: false,
        diagnostic_only,
        reason_codes: stable_reason_codes(&reason_codes),
    }
}

fn load_external_artifacts(paths: &[String]) -> Result<Vec<LoadedExternalReference>, String> {
    let mut loaded = Vec::new();
    for path in paths {
        let text = fs::read_to_string(Path::new(path)).map_err(|err| err.to_string())?;
        let parsed: Value = serde_json::from_str(&text).map_err(|err| err.to_string())?;
        let values = if let Some(items) = parsed.as_array() {
            items.clone()
        } else if let Some(items) = ["external_predictions", "predictions", "rows", "records"]
            .iter()
            .find_map(|key| parsed.get(key).and_then(Value::as_array))
        {
            items.clone()
        } else {
            vec![parsed]
        };
        for value in values {
            let symbol = value
                .get("symbol")
                .and_then(Value::as_str)
                .ok_or_else(|| "external prediction reference is missing symbol".to_string())?;
            let timestamp_ms = value
                .get("timestamp_ms")
                .and_then(Value::as_u64)
                .ok_or_else(|| {
                    "external prediction reference is missing timestamp_ms".to_string()
                })?;
            let external_action = value
                .get("external_action")
                .and_then(Value::as_str)
                .map(str::to_string)
                .or_else(|| {
                    value
                        .get("action")
                        .and_then(Value::as_str)
                        .map(str::to_string)
                });
            let external_p_win = value
                .get("external_p_win")
                .and_then(Value::as_f64)
                .or_else(|| value.get("p_win").and_then(Value::as_f64));
            let external_confidence = value
                .get("external_confidence")
                .and_then(Value::as_f64)
                .or_else(|| value.get("confidence").and_then(Value::as_f64));
            let prediction_schema_valid = external_action.is_some()
                && external_p_win
                    .map(|value| (0.0..=1.0).contains(&value))
                    .unwrap_or(true)
                && external_confidence
                    .map(|value| (0.0..=1.0).contains(&value))
                    .unwrap_or(true);
            loaded.push(LoadedExternalReference {
                symbol: normalize_symbol(symbol),
                timestamp_ms,
                horizon_bars: value
                    .get("horizon_bars")
                    .and_then(Value::as_u64)
                    .map(|value| value as usize),
                reference: super::committee_outcome_reference::CommitteeExternalReference {
                    external_action,
                    external_p_win,
                    external_confidence,
                    prediction_schema_valid,
                    reason_codes: vec![ReasonCode::CommitteeOutcomeReferenceBuilt],
                },
            });
        }
    }
    loaded.sort_by(|left, right| {
        left.symbol
            .cmp(&right.symbol)
            .then(left.timestamp_ms.cmp(&right.timestamp_ms))
            .then(left.horizon_bars.cmp(&right.horizon_bars))
    });
    Ok(loaded)
}

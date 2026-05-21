use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_reason_codes};

use super::candle_coverage_match::{
    CandleCoverageMatchOptions, CandleCoverageMatchStatus, build_candle_coverage_match_computation,
};
use super::comparable_committee_evidence::{
    ComparableCommitteeEvidenceBundle, ComparableCommitteeEvidenceConfig,
    ComparableEvidenceSourceClass,
};
use super::official_candle_coverage_pack::{
    OfficialCandleCoveragePack, OfficialCandleSeriesSourceClass, load_pack_from_path_or_config,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ComparableEvidenceBackfillConfig {
    pub backfill_id: String,
    #[serde(default)]
    pub comparable_evidence_bundle_paths: Vec<String>,
    #[serde(default)]
    pub official_candle_coverage_pack_paths: Vec<String>,
    #[serde(default)]
    pub scenario_pack_paths: Vec<String>,
    #[serde(default = "default_output_root")]
    pub output_root: String,
    #[serde(default = "default_true")]
    pub require_official_for_official_backfill: bool,
    #[serde(default)]
    pub backfill_outcome_reference_available: bool,
    #[serde(default)]
    pub backfill_no_trade_counterfactual_available: bool,
    #[serde(default)]
    pub backfill_risk_denied_counterfactual_available: bool,
    #[serde(default)]
    pub backfill_baseline_reference_available: bool,
    #[serde(default)]
    pub allow_diagnostic_backfill: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ComparableEvidenceBackfillStatus {
    BackfillImproved,
    OfficialBackfillImproved,
    DiagnosticBackfillOnly,
    NoBackfillPossible,
    StillMissingCandles,
    #[default]
    StillMaterializationWeak,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ComparableEvidenceBackfillReport {
    pub backfill_id: String,
    pub input_rows: usize,
    pub output_rows: usize,
    pub rows_with_new_candle_match: usize,
    pub rows_with_new_official_ready_match: usize,
    pub rows_upgraded_from_missing_candles: usize,
    pub rows_still_missing_candles: usize,
    pub rows_still_summary_derived: usize,
    pub rows_still_diagnostic_only: usize,
    pub source_summary: String,
    pub backfill_status: ComparableEvidenceBackfillStatus,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ComparableEvidenceBackfillResult {
    pub bundle: ComparableCommitteeEvidenceBundle,
    pub report: ComparableEvidenceBackfillReport,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ComparableEvidenceBackfillRunner;

impl Default for ComparableEvidenceBackfillConfig {
    fn default() -> Self {
        Self {
            backfill_id: "comparable-evidence-backfill".to_string(),
            comparable_evidence_bundle_paths: Vec::new(),
            official_candle_coverage_pack_paths: Vec::new(),
            scenario_pack_paths: Vec::new(),
            output_root: default_output_root(),
            require_official_for_official_backfill: true,
            backfill_outcome_reference_available: false,
            backfill_no_trade_counterfactual_available: false,
            backfill_risk_denied_counterfactual_available: false,
            backfill_baseline_reference_available: false,
            allow_diagnostic_backfill: false,
            reason_codes: vec![ReasonCode::DeterministicPath],
        }
    }
}

impl ComparableEvidenceBackfillConfig {
    pub fn from_toml_str(input: &str) -> Result<Self, String> {
        toml::from_str(input).map_err(|err| err.to_string())
    }

    pub fn from_toml_path(path: &Path) -> Result<Self, String> {
        let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
        Self::from_toml_str(&text)
    }

    pub fn to_toml_string(&self) -> Result<String, String> {
        toml::to_string_pretty(self).map_err(|err| err.to_string())
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.backfill_id.trim().is_empty() {
            return Err("comparable evidence backfill id must not be empty".to_string());
        }
        if self
            .all_paths()
            .iter()
            .chain(std::iter::once(&self.output_root))
            .any(|path| path.contains("://"))
        {
            return Err("comparable evidence backfill paths must be local".to_string());
        }
        Ok(())
    }

    pub fn output_dir(&self) -> PathBuf {
        PathBuf::from(&self.output_root).join(&self.backfill_id)
    }

    pub fn all_paths(&self) -> Vec<String> {
        self.comparable_evidence_bundle_paths
            .iter()
            .chain(self.official_candle_coverage_pack_paths.iter())
            .chain(self.scenario_pack_paths.iter())
            .cloned()
            .collect()
    }

    pub fn comparable_config(&self) -> ComparableCommitteeEvidenceConfig {
        ComparableCommitteeEvidenceConfig {
            comparable_id: self.backfill_id.clone(),
            output_root: self.output_root.clone(),
            ..ComparableCommitteeEvidenceConfig::default()
        }
    }
}

impl ComparableEvidenceBackfillRunner {
    pub fn run(
        &self,
        config: &ComparableEvidenceBackfillConfig,
    ) -> Result<ComparableEvidenceBackfillReport, String> {
        self.run_bundle(config).map(|result| result.report)
    }

    pub fn run_bundle(
        &self,
        config: &ComparableEvidenceBackfillConfig,
    ) -> Result<ComparableEvidenceBackfillResult, String> {
        config.validate()?;
        let packs = config
            .official_candle_coverage_pack_paths
            .iter()
            .map(|path| load_pack_from_path_or_config(path))
            .collect::<Result<Vec<_>, _>>()?;
        let pack = merge_packs(&config.backfill_id, packs);
        let mut rows = Vec::new();
        for path in &config.comparable_evidence_bundle_paths {
            rows.extend(ComparableCommitteeEvidenceBundle::from_json_path(Path::new(path))?.rows);
        }
        let input_rows = rows.len();
        let computation = build_candle_coverage_match_computation(
            &rows,
            &pack,
            &CandleCoverageMatchOptions::default(),
        );
        let mut rows_with_new_candle_match = 0usize;
        let mut rows_with_new_official_ready_match = 0usize;
        let mut rows_upgraded_from_missing_candles = 0usize;

        let updated_rows =
            rows.into_iter()
                .map(|mut row| {
                    let matched = computation.match_report.matches.iter().find(|entry| {
                        entry.comparable_row_id.as_deref() == Some(row.row_id.as_str())
                    });
                    if let Some(matched) = matched {
                        let can_apply = match matched.match_status {
                            CandleCoverageMatchStatus::Matched => true,
                            CandleCoverageMatchStatus::MatchedDiagnosticOnly => {
                                config.allow_diagnostic_backfill || !matched.diagnostic_only
                            }
                            CandleCoverageMatchStatus::SourceNotEligible => {
                                config.allow_diagnostic_backfill
                            }
                            _ => false,
                        };
                        if can_apply {
                            if !row.candle_coverage_available {
                                rows_with_new_candle_match += 1;
                                rows_upgraded_from_missing_candles += 1;
                            }
                            if matched.official_ready_match {
                                rows_with_new_official_ready_match += 1;
                            }
                            row.candle_coverage_available = true;
                            row.matched_candle_series_id = matched.candle_series_id.clone();
                            row.candle_match_status = Some(format!("{:?}", matched.match_status));
                            row.candle_official_ready_match = matched.official_ready_match
                                && (!config.require_official_for_official_backfill
                                    || row.source_class
                                        == ComparableEvidenceSourceClass::OfficialNonCrypto);
                            row.candle_benchmark_ready_match = matched.benchmark_ready_match;
                            row.candle_diagnostic_only = matched.diagnostic_only;
                            row.reason_codes = stable_reason_codes(
                                &row.reason_codes
                                    .into_iter()
                                    .chain([ReasonCode::OfficialCandleCoverageBuilt])
                                    .collect::<Vec<_>>(),
                            );
                        }
                    }
                    row
                })
                .collect::<Vec<_>>();

        let bundle =
            ComparableCommitteeEvidenceBundle::from_rows(&config.comparable_config(), updated_rows);
        let rows_still_missing_candles = bundle
            .rows
            .iter()
            .filter(|row| !row.candle_coverage_available)
            .count();
        let rows_still_summary_derived =
            bundle.rows.iter().filter(|row| row.summary_derived).count();
        let rows_still_diagnostic_only = bundle
            .rows
            .iter()
            .filter(|row| row.diagnostic_only || row.candle_diagnostic_only)
            .count();
        let source_summary = format!(
            "official_non_crypto={};crypto_only={};controlled={};yfinance={};fixture={}",
            bundle.non_crypto_official_rows,
            bundle.crypto_only_rows,
            bundle.controlled_rows,
            bundle.yfinance_rows,
            bundle.fixture_rows,
        );
        let backfill_status = determine_backfill_status(
            &bundle,
            rows_with_new_candle_match,
            rows_with_new_official_ready_match,
            rows_still_missing_candles,
        );
        let report = ComparableEvidenceBackfillReport {
            backfill_id: config.backfill_id.clone(),
            input_rows,
            output_rows: bundle.rows.len(),
            rows_with_new_candle_match,
            rows_with_new_official_ready_match,
            rows_upgraded_from_missing_candles,
            rows_still_missing_candles,
            rows_still_summary_derived,
            rows_still_diagnostic_only,
            source_summary,
            backfill_status,
            reason_codes: stable_reason_codes(
                &config
                    .reason_codes
                    .iter()
                    .cloned()
                    .chain([
                        ReasonCode::OfficialCandleCoverageBuilt,
                        ReasonCode::DeterministicPath,
                    ])
                    .collect::<Vec<_>>(),
            ),
        };
        Ok(ComparableEvidenceBackfillResult { bundle, report })
    }
}

impl ComparableEvidenceBackfillReport {
    pub fn to_text(&self) -> String {
        [
            format!("backfill_id={}", self.backfill_id),
            format!("input_rows={}", self.input_rows),
            format!("output_rows={}", self.output_rows),
            format!(
                "rows_with_new_candle_match={}",
                self.rows_with_new_candle_match
            ),
            format!(
                "rows_with_new_official_ready_match={}",
                self.rows_with_new_official_ready_match
            ),
            format!(
                "rows_upgraded_from_missing_candles={}",
                self.rows_upgraded_from_missing_candles
            ),
            format!(
                "rows_still_missing_candles={}",
                self.rows_still_missing_candles
            ),
            format!(
                "rows_still_summary_derived={}",
                self.rows_still_summary_derived
            ),
            format!(
                "rows_still_diagnostic_only={}",
                self.rows_still_diagnostic_only
            ),
            format!("source_summary={}", self.source_summary),
            format!("backfill_status={:?}", self.backfill_status),
        ]
        .join("\n")
    }
}

impl ComparableEvidenceBackfillResult {
    pub fn write_to_dir(&self, output_dir: &Path) -> Result<PathBuf, String> {
        fs::create_dir_all(output_dir).map_err(|err| err.to_string())?;
        self.bundle.write_to_dir(output_dir)?;
        fs::write(
            output_dir.join("comparable_backfill.txt"),
            self.report.to_text(),
        )
        .map_err(|err| err.to_string())?;
        let json_path = output_dir.join("comparable_backfill_report.json");
        fs::write(
            &json_path,
            serde_json::to_string_pretty(&self.report).map_err(|err| err.to_string())?,
        )
        .map_err(|err| err.to_string())?;
        Ok(json_path)
    }
}

fn merge_packs(
    pack_id: &str,
    packs: Vec<OfficialCandleCoveragePack>,
) -> OfficialCandleCoveragePack {
    let mut descriptors = packs
        .into_iter()
        .flat_map(|pack| pack.descriptors)
        .collect::<Vec<_>>();
    descriptors.sort_by(|left, right| {
        left.candle_series_id
            .cmp(&right.candle_series_id)
            .then(left.path.cmp(&right.path))
    });
    descriptors.dedup_by(|left, right| {
        left.candle_series_id == right.candle_series_id && left.path == right.path
    });
    let total_rows = descriptors
        .iter()
        .map(|descriptor| descriptor.row_count)
        .sum();
    let total_symbols = descriptors
        .iter()
        .map(|descriptor| descriptor.normalized_symbol.clone())
        .collect::<std::collections::BTreeSet<_>>()
        .len();
    let total_timeframes = descriptors
        .iter()
        .map(|descriptor| descriptor.timeframe.clone())
        .collect::<std::collections::BTreeSet<_>>()
        .len();
    let storage_bytes = descriptors
        .iter()
        .map(|descriptor| descriptor.storage_bytes)
        .sum();
    let official_non_crypto_series = descriptors
        .iter()
        .filter(|descriptor| {
            descriptor.source_class == OfficialCandleSeriesSourceClass::OfficialNonCrypto
        })
        .cloned()
        .collect::<Vec<_>>();
    let official_crypto_series = descriptors
        .iter()
        .filter(|descriptor| {
            descriptor.source_class == OfficialCandleSeriesSourceClass::OfficialCryptoOnly
        })
        .cloned()
        .collect::<Vec<_>>();
    let controlled_series = descriptors
        .iter()
        .filter(|descriptor| {
            descriptor.source_class == OfficialCandleSeriesSourceClass::ControlledDiagnostic
        })
        .cloned()
        .collect::<Vec<_>>();
    let yfinance_series = descriptors
        .iter()
        .filter(|descriptor| {
            descriptor.source_class == OfficialCandleSeriesSourceClass::YFinanceResearch
        })
        .cloned()
        .collect::<Vec<_>>();
    let fixture_series = descriptors
        .iter()
        .filter(|descriptor| {
            matches!(
                descriptor.source_class,
                OfficialCandleSeriesSourceClass::FixtureArchitectureTest
                    | OfficialCandleSeriesSourceClass::SyntheticTest
            )
        })
        .cloned()
        .collect::<Vec<_>>();
    let unknown_series = descriptors
        .iter()
        .filter(|descriptor| descriptor.source_class == OfficialCandleSeriesSourceClass::Unknown)
        .cloned()
        .collect::<Vec<_>>();
    let readiness_eligible_series_count = descriptors
        .iter()
        .filter(|descriptor| descriptor.official_readiness_eligible)
        .count();
    let benchmark_eligible_series_count = descriptors
        .iter()
        .filter(|descriptor| descriptor.benchmark_eligible)
        .count();
    OfficialCandleCoveragePack {
        pack_id: pack_id.to_string(),
        descriptors,
        official_non_crypto_series,
        official_crypto_series,
        controlled_series,
        yfinance_series,
        fixture_series,
        unknown_series,
        total_rows,
        total_symbols,
        total_timeframes,
        storage_bytes,
        readiness_eligible_series_count,
        benchmark_eligible_series_count,
        warnings: Vec::new(),
        reason_codes: stable_reason_codes(&[ReasonCode::OfficialCandleCoverageBuilt]),
    }
}

fn determine_backfill_status(
    bundle: &ComparableCommitteeEvidenceBundle,
    rows_with_new_candle_match: usize,
    rows_with_new_official_ready_match: usize,
    rows_still_missing_candles: usize,
) -> ComparableEvidenceBackfillStatus {
    if rows_with_new_official_ready_match > 0 {
        ComparableEvidenceBackfillStatus::OfficialBackfillImproved
    } else if rows_with_new_candle_match > 0
        && bundle
            .rows
            .iter()
            .all(|row| row.diagnostic_only || row.candle_diagnostic_only)
    {
        ComparableEvidenceBackfillStatus::DiagnosticBackfillOnly
    } else if rows_with_new_candle_match > 0 {
        ComparableEvidenceBackfillStatus::BackfillImproved
    } else if rows_still_missing_candles > 0 {
        ComparableEvidenceBackfillStatus::StillMissingCandles
    } else if bundle.summary_derived_count > 0 {
        ComparableEvidenceBackfillStatus::StillMaterializationWeak
    } else {
        ComparableEvidenceBackfillStatus::NoBackfillPossible
    }
}

fn default_output_root() -> String {
    "target/soma_comparable_evidence_backfill".to_string()
}

fn default_true() -> bool {
    true
}

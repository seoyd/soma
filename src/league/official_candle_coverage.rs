use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::CandleSeries;
use crate::core::{ReasonCode, stable_reason_codes};

use super::candle_alignment::{CandleAligner, CandleAlignmentOverallStatus, CandleAlignmentStatus};
use super::committee_counterfactual_builder::load_local_candle_series_map;
use super::committee_reference_pack::CommitteeReferencePackConfig;
use super::committee_scenario_loader::CommitteeScenarioRow;
use super::official_evidence_replication::OfficialEvidenceReplicationConfig;
use super::official_replication_inventory::{
    OfficialReplicationArtifactInventory, OfficialReplicationArtifactKind,
};
use super::official_row_injection::{OfficialEvidenceBoundary, classify_row_boundary};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum OfficialCandleCoverageStatus {
    HealthyOfficialCandleCoverage,
    MissingOfficialCandles,
    MissingFutureWindow,
    TimestampAlignmentWeak,
    SymbolMismatch,
    TimeframeMismatch,
    BadDataQuality,
    CryptoOnlyCoverage,
    ControlledOnlyCoverage,
    InsufficientCoverage,
    #[default]
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OfficialCandleCoverageReport {
    pub official_rows: usize,
    pub rows_with_candles: usize,
    pub rows_with_future_window: usize,
    pub rows_no_lookahead_safe: usize,
    pub missing_candle_rows: usize,
    pub missing_future_window_rows: usize,
    pub timestamp_mismatch_rows: usize,
    pub symbol_mismatch_rows: usize,
    pub timeframe_mismatch_rows: usize,
    pub gap_rows: usize,
    pub duplicate_timestamp_rows: usize,
    pub coverage_status: OfficialCandleCoverageStatus,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct OfficialCandleCoverageRunner;

impl OfficialCandleCoverageRunner {
    pub fn run(
        &self,
        rows: &[CommitteeScenarioRow],
        candle_series_paths: &[String],
    ) -> Result<OfficialCandleCoverageReport, String> {
        let official_rows = rows
            .iter()
            .filter(|row| {
                matches!(
                    classify_row_boundary(row),
                    OfficialEvidenceBoundary::OfficialNonCrypto
                )
            })
            .count();
        if rows.is_empty() {
            return Ok(OfficialCandleCoverageReport {
                official_rows,
                rows_with_candles: 0,
                rows_with_future_window: 0,
                rows_no_lookahead_safe: 0,
                missing_candle_rows: 0,
                missing_future_window_rows: 0,
                timestamp_mismatch_rows: 0,
                symbol_mismatch_rows: 0,
                timeframe_mismatch_rows: 0,
                gap_rows: 0,
                duplicate_timestamp_rows: 0,
                coverage_status: OfficialCandleCoverageStatus::InsufficientCoverage,
                reason_codes: stable_reason_codes(&[
                    ReasonCode::OfficialCandleCoverageBuilt,
                    ReasonCode::EvidenceStillInsufficient,
                ]),
            });
        }
        let mut config = CommitteeReferencePackConfig::default();
        config.reference_pack_id = "official_candle_coverage".to_string();
        config.require_exact_horizon_match = true;
        config.build_baseline_references = false;
        config.build_no_trade_counterfactuals = false;
        config.build_risk_denied_counterfactuals = false;
        config.build_triple_barrier_outcomes = false;
        config.candle_series_paths = candle_series_paths.to_vec();
        let series_map = load_local_candle_series_map(candle_series_paths)?;
        let alignment = CandleAligner::default().align_rows(rows, &series_map, &config);
        let rows_with_candles = alignment.matched_count;
        let rows_with_future_window = alignment
            .records
            .iter()
            .filter(|record| {
                matches!(
                    record.status,
                    CandleAlignmentStatus::MatchedExact
                        | CandleAlignmentStatus::MatchedWithTolerance
                ) && record.future_window_end_index.is_some()
            })
            .count();
        let rows_no_lookahead_safe = alignment
            .records
            .iter()
            .filter(|record| record.no_lookahead_safe)
            .count();
        let missing_candle_rows = alignment.missing_series_count;
        let missing_future_window_rows = alignment.insufficient_future_bars_count;
        let timestamp_mismatch_rows = alignment.missing_timestamp_count;
        let symbol_mismatch_rows = alignment.wrong_symbol_count;
        let timeframe_mismatch_rows = alignment
            .records
            .iter()
            .filter(|record| record.status == CandleAlignmentStatus::WrongHorizon)
            .count();
        let gap_rows = alignment
            .records
            .iter()
            .filter(|record| record.status == CandleAlignmentStatus::GapDetected)
            .count();
        let duplicate_timestamp_rows = alignment
            .records
            .iter()
            .filter(|record| record.status == CandleAlignmentStatus::DuplicateTimestamp)
            .count();
        let boundary_status = if official_rows == 0 {
            if rows
                .iter()
                .any(|row| classify_row_boundary(row) == OfficialEvidenceBoundary::Controlled)
            {
                Some(OfficialCandleCoverageStatus::ControlledOnlyCoverage)
            } else if rows.iter().any(|row| {
                classify_row_boundary(row) == OfficialEvidenceBoundary::OfficialCryptoOnly
            }) {
                Some(OfficialCandleCoverageStatus::CryptoOnlyCoverage)
            } else {
                None
            }
        } else {
            None
        };
        let coverage_status = boundary_status.unwrap_or_else(|| match alignment.alignment_status {
            CandleAlignmentOverallStatus::NeedMoreCandleData => {
                if timeframe_mismatch_rows > 0 {
                    OfficialCandleCoverageStatus::TimeframeMismatch
                } else {
                    OfficialCandleCoverageStatus::MissingOfficialCandles
                }
            }
            CandleAlignmentOverallStatus::NeedLongerFutureWindows => {
                OfficialCandleCoverageStatus::MissingFutureWindow
            }
            CandleAlignmentOverallStatus::NeedBetterTimestampAlignment => {
                if symbol_mismatch_rows > 0 {
                    OfficialCandleCoverageStatus::SymbolMismatch
                } else if timeframe_mismatch_rows > 0 {
                    OfficialCandleCoverageStatus::TimeframeMismatch
                } else {
                    OfficialCandleCoverageStatus::TimestampAlignmentWeak
                }
            }
            CandleAlignmentOverallStatus::BadDataQuality => {
                OfficialCandleCoverageStatus::BadDataQuality
            }
            CandleAlignmentOverallStatus::HealthyAlignment => {
                if rows_no_lookahead_safe < rows_with_future_window {
                    OfficialCandleCoverageStatus::BadDataQuality
                } else {
                    OfficialCandleCoverageStatus::HealthyOfficialCandleCoverage
                }
            }
            CandleAlignmentOverallStatus::DiagnosticOnly
            | CandleAlignmentOverallStatus::Unknown => {
                OfficialCandleCoverageStatus::InsufficientCoverage
            }
        });
        Ok(OfficialCandleCoverageReport {
            official_rows,
            rows_with_candles,
            rows_with_future_window,
            rows_no_lookahead_safe,
            missing_candle_rows,
            missing_future_window_rows,
            timestamp_mismatch_rows,
            symbol_mismatch_rows,
            timeframe_mismatch_rows,
            gap_rows,
            duplicate_timestamp_rows,
            coverage_status,
            reason_codes: stable_reason_codes(&[
                ReasonCode::OfficialCandleCoverageBuilt,
                ReasonCode::CandleAlignmentBuilt,
            ]),
        })
    }
}

impl OfficialCandleCoverageReport {
    pub fn to_text(&self) -> String {
        [
            format!("official_rows={}", self.official_rows),
            format!("rows_with_candles={}", self.rows_with_candles),
            format!("rows_with_future_window={}", self.rows_with_future_window),
            format!("rows_no_lookahead_safe={}", self.rows_no_lookahead_safe),
            format!("missing_candle_rows={}", self.missing_candle_rows),
            format!(
                "missing_future_window_rows={}",
                self.missing_future_window_rows
            ),
            format!("timestamp_mismatch_rows={}", self.timestamp_mismatch_rows),
            format!("symbol_mismatch_rows={}", self.symbol_mismatch_rows),
            format!("timeframe_mismatch_rows={}", self.timeframe_mismatch_rows),
            format!("gap_rows={}", self.gap_rows),
            format!("duplicate_timestamp_rows={}", self.duplicate_timestamp_rows),
            format!("coverage_status={:?}", self.coverage_status),
        ]
        .join("\n")
    }
}

pub fn materialize_official_candle_series(
    config: &OfficialEvidenceReplicationConfig,
    inventory: &OfficialReplicationArtifactInventory,
) -> Result<Vec<String>, String> {
    let output_dir = config.output_dir().join("candle_series");
    fs::create_dir_all(&output_dir).map_err(|err| err.to_string())?;
    let mut written = BTreeSet::new();
    for descriptor in inventory.descriptors.iter().filter(|descriptor| {
        descriptor.artifact_kind == OfficialReplicationArtifactKind::OfficialCanonicalCsv
    }) {
        let json_path = write_candle_series_from_csv(&descriptor.path, &output_dir)?;
        written.insert(json_path.display().to_string());
    }
    Ok(written.into_iter().collect())
}

pub fn load_candle_series_from_paths(paths: &[String]) -> Result<Vec<CandleSeries>, String> {
    let loaded = load_local_candle_series_map(paths)?;
    Ok(loaded.into_values().collect())
}

fn write_candle_series_from_csv(path: &str, output_dir: &Path) -> Result<PathBuf, String> {
    let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
    let mut lines = text.lines().filter(|line| !line.trim().is_empty());
    let header = lines
        .next()
        .ok_or_else(|| format!("{path} is empty"))?
        .split(',')
        .map(|value| value.trim().to_ascii_lowercase())
        .collect::<Vec<_>>();
    let timestamp_index = header
        .iter()
        .position(|value| value == "timestamp" || value == "timestamp_ms")
        .ok_or_else(|| format!("{path} missing timestamp column"))?;
    let open_index = header
        .iter()
        .position(|value| value == "open")
        .ok_or_else(|| format!("{path} missing open column"))?;
    let high_index = header
        .iter()
        .position(|value| value == "high")
        .ok_or_else(|| format!("{path} missing high column"))?;
    let low_index = header
        .iter()
        .position(|value| value == "low")
        .ok_or_else(|| format!("{path} missing low column"))?;
    let close_index = header
        .iter()
        .position(|value| value == "close")
        .ok_or_else(|| format!("{path} missing close column"))?;
    let volume_index = header.iter().position(|value| value == "volume");
    let symbol = Path::new(path)
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("UNKNOWN")
        .trim_end_matches("_candles")
        .to_string();
    let candles = lines
        .enumerate()
        .filter_map(|(_, line)| {
            let columns = line.split(',').map(|value| value.trim()).collect::<Vec<_>>();
            (columns.len() > close_index).then(|| {
                json!({
                    "timestamp_ms": normalize_timestamp_ms(columns[timestamp_index].parse::<u64>().unwrap_or(0)),
                    "open": columns[open_index].parse::<f64>().unwrap_or(0.0),
                    "high": columns[high_index].parse::<f64>().unwrap_or(0.0),
                    "low": columns[low_index].parse::<f64>().unwrap_or(0.0),
                    "close": columns[close_index].parse::<f64>().unwrap_or(0.0),
                    "volume": volume_index.and_then(|column| columns.get(column)).and_then(|value| value.parse::<f64>().ok()).unwrap_or(0.0),
                    "spread_bps": 4.0,
                })
            })
        })
        .collect::<Vec<_>>();
    let output_path = output_dir.join(format!("{}_candles.json", symbol.to_ascii_lowercase()));
    fs::write(
        &output_path,
        serde_json::to_string_pretty(&json!({
            "symbol": symbol,
            "timeframe": "OneDay",
            "candles": candles,
        }))
        .map_err(|err| err.to_string())?,
    )
    .map_err(|err| err.to_string())?;
    Ok(output_path)
}

fn normalize_timestamp_ms(value: u64) -> u64 {
    if value == 0 {
        1_700_000_000_000
    } else if value < 1_000_000_000_000 {
        value.saturating_mul(1000)
    } else {
        value
    }
}

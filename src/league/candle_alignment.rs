use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::CandleSeries;
use crate::core::{ReasonCode, stable_reason_codes};

use super::committee_counterfactual_builder::{horizon_bars_for_row, normalize_symbol};
use super::committee_reference_pack::CommitteeReferencePackConfig;
use super::committee_scenario_loader::CommitteeScenarioRow;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum CandleAlignmentStatus {
    MatchedExact,
    MatchedWithTolerance,
    MissingCandleSeries,
    MissingTimestamp,
    WrongSymbol,
    WrongHorizon,
    GapDetected,
    DuplicateTimestamp,
    InsufficientFutureBars,
    BadDataQuality,
    RejectedNoLookahead,
    #[default]
    Unknown,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum CandleAlignmentOverallStatus {
    HealthyAlignment,
    NeedMoreCandleData,
    NeedBetterTimestampAlignment,
    NeedLongerFutureWindows,
    BadDataQuality,
    DiagnosticOnly,
    #[default]
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CandleAlignmentRecord {
    pub scenario_row_id: String,
    pub symbol: String,
    pub timestamp_ms: u64,
    pub horizon_bars: usize,
    #[serde(default)]
    pub candle_series_id: Option<String>,
    #[serde(default)]
    pub matched_start_index: Option<usize>,
    #[serde(default)]
    pub matched_end_index: Option<usize>,
    #[serde(default)]
    pub future_window_start_index: Option<usize>,
    #[serde(default)]
    pub future_window_end_index: Option<usize>,
    pub status: CandleAlignmentStatus,
    pub no_lookahead_safe: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CandleAlignmentReport {
    pub records: Vec<CandleAlignmentRecord>,
    pub matched_count: usize,
    pub unmatched_count: usize,
    pub exact_match_count: usize,
    pub tolerance_match_count: usize,
    pub missing_series_count: usize,
    pub missing_timestamp_count: usize,
    pub wrong_symbol_count: usize,
    pub insufficient_future_bars_count: usize,
    pub no_lookahead_rejected_count: usize,
    pub alignment_status: CandleAlignmentOverallStatus,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CandleAligner;

impl CandleAligner {
    pub fn align_rows(
        &self,
        rows: &[CommitteeScenarioRow],
        series_map: &BTreeMap<String, CandleSeries>,
        config: &CommitteeReferencePackConfig,
    ) -> CandleAlignmentReport {
        let mut records = rows
            .iter()
            .map(|row| self.align_row(row, series_map, config))
            .collect::<Vec<_>>();
        records.sort_by(|left, right| left.scenario_row_id.cmp(&right.scenario_row_id));
        let matched_count = records
            .iter()
            .filter(|record| {
                matches!(
                    record.status,
                    CandleAlignmentStatus::MatchedExact
                        | CandleAlignmentStatus::MatchedWithTolerance
                )
            })
            .count();
        let unmatched_count = records.len().saturating_sub(matched_count);
        let exact_match_count = records
            .iter()
            .filter(|record| record.status == CandleAlignmentStatus::MatchedExact)
            .count();
        let tolerance_match_count = records
            .iter()
            .filter(|record| record.status == CandleAlignmentStatus::MatchedWithTolerance)
            .count();
        let missing_series_count = records
            .iter()
            .filter(|record| record.status == CandleAlignmentStatus::MissingCandleSeries)
            .count();
        let missing_timestamp_count = records
            .iter()
            .filter(|record| record.status == CandleAlignmentStatus::MissingTimestamp)
            .count();
        let wrong_symbol_count = records
            .iter()
            .filter(|record| record.status == CandleAlignmentStatus::WrongSymbol)
            .count();
        let insufficient_future_bars_count = records
            .iter()
            .filter(|record| record.status == CandleAlignmentStatus::InsufficientFutureBars)
            .count();
        let no_lookahead_rejected_count = records
            .iter()
            .filter(|record| record.status == CandleAlignmentStatus::RejectedNoLookahead)
            .count();
        let alignment_status = determine_alignment_status(
            &records,
            matched_count,
            tolerance_match_count,
            insufficient_future_bars_count,
            no_lookahead_rejected_count,
        );
        CandleAlignmentReport {
            records,
            matched_count,
            unmatched_count,
            exact_match_count,
            tolerance_match_count,
            missing_series_count,
            missing_timestamp_count,
            wrong_symbol_count,
            insufficient_future_bars_count,
            no_lookahead_rejected_count,
            alignment_status,
            reason_codes: vec![ReasonCode::CandleAlignmentBuilt],
        }
    }

    fn align_row(
        &self,
        row: &CommitteeScenarioRow,
        series_map: &BTreeMap<String, CandleSeries>,
        config: &CommitteeReferencePackConfig,
    ) -> CandleAlignmentRecord {
        let horizon_bars = horizon_bars_for_row(row, config.default_horizon_bars);
        let mut reason_codes = row.reason_codes.clone();
        let normalized_symbol = normalize_symbol(&row.symbol);
        if config.require_exact_horizon_match && horizon_bars != config.default_horizon_bars {
            reason_codes.push(ReasonCode::HorizonFiltered);
            return record_for(
                row,
                horizon_bars,
                CandleAlignmentStatus::WrongHorizon,
                false,
                reason_codes,
            );
        }
        let Some(series) = series_map.get(&normalized_symbol) else {
            let wrong_symbol = series_map.values().any(|series| {
                timestamp_exists(series, row.timestamp_ms, config.timestamp_tolerance_ms)
            });
            reason_codes.push(ReasonCode::MissingRealLocalData);
            return record_for(
                row,
                horizon_bars,
                if wrong_symbol {
                    CandleAlignmentStatus::WrongSymbol
                } else {
                    CandleAlignmentStatus::MissingCandleSeries
                },
                false,
                reason_codes,
            );
        };
        if !series.candles.iter().all(|candle| {
            candle.open.is_finite()
                && candle.high.is_finite()
                && candle.low.is_finite()
                && candle.close.is_finite()
                && candle.open > 0.0
                && candle.high > 0.0
                && candle.low > 0.0
                && candle.close > 0.0
                && candle.high >= candle.low
        }) {
            reason_codes.push(ReasonCode::DataQualityTooLow);
            return record_with_indices(
                row,
                horizon_bars,
                Some(series.symbol.clone()),
                None,
                None,
                None,
                None,
                CandleAlignmentStatus::BadDataQuality,
                false,
                reason_codes,
            );
        }
        let matches = series
            .candles
            .iter()
            .enumerate()
            .filter_map(|(index, candle)| {
                let distance = candle.timestamp_ms.max(row.timestamp_ms)
                    - candle.timestamp_ms.min(row.timestamp_ms);
                (distance <= config.timestamp_tolerance_ms).then_some((
                    index,
                    distance,
                    candle.timestamp_ms,
                ))
            })
            .collect::<Vec<_>>();
        if matches.is_empty() {
            reason_codes.push(ReasonCode::StaleTimestamp);
            return record_with_indices(
                row,
                horizon_bars,
                Some(series.symbol.clone()),
                None,
                None,
                None,
                None,
                CandleAlignmentStatus::MissingTimestamp,
                false,
                reason_codes,
            );
        }
        let min_distance = matches
            .iter()
            .map(|(_, distance, _)| *distance)
            .min()
            .unwrap_or(u64::MAX);
        let closest = matches
            .iter()
            .filter(|(_, distance, _)| *distance == min_distance)
            .collect::<Vec<_>>();
        if closest.len() > 1 {
            reason_codes.push(ReasonCode::DuplicateTimestampDetected);
            return record_with_indices(
                row,
                horizon_bars,
                Some(series.symbol.clone()),
                None,
                None,
                None,
                None,
                CandleAlignmentStatus::DuplicateTimestamp,
                false,
                reason_codes,
            );
        }
        let (matched_start_index, distance, matched_timestamp) = *closest[0];
        let future_window_start_index = matched_start_index + 1;
        let future_window_end_index = matched_start_index + horizon_bars;
        if future_window_end_index >= series.len() || horizon_bars == 0 {
            reason_codes.push(ReasonCode::InsufficientBars);
            return record_with_indices(
                row,
                horizon_bars,
                Some(series.symbol.clone()),
                Some(matched_start_index),
                Some(matched_start_index),
                Some(future_window_start_index),
                Some(future_window_end_index.min(series.len().saturating_sub(1))),
                CandleAlignmentStatus::InsufficientFutureBars,
                false,
                reason_codes,
            );
        }
        if window_has_gap(series, matched_start_index, future_window_end_index) {
            reason_codes.push(ReasonCode::GapDetected);
            return record_with_indices(
                row,
                horizon_bars,
                Some(series.symbol.clone()),
                Some(matched_start_index),
                Some(matched_start_index),
                Some(future_window_start_index),
                Some(future_window_end_index),
                CandleAlignmentStatus::GapDetected,
                false,
                reason_codes,
            );
        }
        let no_lookahead_safe = matched_timestamp <= row.timestamp_ms
            && future_window_start_index > matched_start_index;
        if config.require_no_lookahead_safe && !no_lookahead_safe {
            reason_codes.push(ReasonCode::RejectedNoLookaheadReference);
            return record_with_indices(
                row,
                horizon_bars,
                Some(series.symbol.clone()),
                Some(matched_start_index),
                Some(matched_start_index),
                Some(future_window_start_index),
                Some(future_window_end_index),
                CandleAlignmentStatus::RejectedNoLookahead,
                false,
                reason_codes,
            );
        }
        let status = if distance == 0 {
            CandleAlignmentStatus::MatchedExact
        } else {
            CandleAlignmentStatus::MatchedWithTolerance
        };
        record_with_indices(
            row,
            horizon_bars,
            Some(series.symbol.clone()),
            Some(matched_start_index),
            Some(matched_start_index),
            Some(future_window_start_index),
            Some(future_window_end_index),
            status,
            no_lookahead_safe,
            reason_codes,
        )
    }
}

impl CandleAlignmentReport {
    pub fn from_json_path(path: &Path) -> Result<Self, String> {
        let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
        serde_json::from_str(&text).map_err(|err| err.to_string())
    }

    pub fn to_json_string(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|err| err.to_string())
    }

    pub fn to_text(&self) -> String {
        let mut lines = vec![
            format!("matched_count={}", self.matched_count),
            format!("unmatched_count={}", self.unmatched_count),
            format!("exact_match_count={}", self.exact_match_count),
            format!("tolerance_match_count={}", self.tolerance_match_count),
            format!("missing_series_count={}", self.missing_series_count),
            format!("missing_timestamp_count={}", self.missing_timestamp_count),
            format!("wrong_symbol_count={}", self.wrong_symbol_count),
            format!(
                "insufficient_future_bars_count={}",
                self.insufficient_future_bars_count
            ),
            format!(
                "no_lookahead_rejected_count={}",
                self.no_lookahead_rejected_count
            ),
            format!("alignment_status={:?}", self.alignment_status),
        ];
        for record in &self.records {
            lines.push(format!(
                "scenario_row_id={};symbol={};timestamp_ms={};horizon_bars={};status={:?};no_lookahead_safe={}",
                record.scenario_row_id,
                record.symbol,
                record.timestamp_ms,
                record.horizon_bars,
                record.status,
                record.no_lookahead_safe,
            ));
        }
        lines.join("\n")
    }

    pub fn write_to_dir(&self, output_dir: &Path) -> Result<PathBuf, String> {
        fs::create_dir_all(output_dir).map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("candle_alignment_report.txt"),
            self.to_text(),
        )
        .map_err(|err| err.to_string())?;
        let json_path = output_dir.join("candle_alignment_report.json");
        fs::write(&json_path, self.to_json_string()?).map_err(|err| err.to_string())?;
        Ok(json_path)
    }
}

fn record_for(
    row: &CommitteeScenarioRow,
    horizon_bars: usize,
    status: CandleAlignmentStatus,
    no_lookahead_safe: bool,
    reason_codes: Vec<ReasonCode>,
) -> CandleAlignmentRecord {
    record_with_indices(
        row,
        horizon_bars,
        None,
        None,
        None,
        None,
        None,
        status,
        no_lookahead_safe,
        reason_codes,
    )
}

#[allow(clippy::too_many_arguments)]
fn record_with_indices(
    row: &CommitteeScenarioRow,
    horizon_bars: usize,
    candle_series_id: Option<String>,
    matched_start_index: Option<usize>,
    matched_end_index: Option<usize>,
    future_window_start_index: Option<usize>,
    future_window_end_index: Option<usize>,
    status: CandleAlignmentStatus,
    no_lookahead_safe: bool,
    reason_codes: Vec<ReasonCode>,
) -> CandleAlignmentRecord {
    CandleAlignmentRecord {
        scenario_row_id: row.scenario_row_id.clone(),
        symbol: normalize_symbol(&row.symbol),
        timestamp_ms: row.timestamp_ms,
        horizon_bars,
        candle_series_id,
        matched_start_index,
        matched_end_index,
        future_window_start_index,
        future_window_end_index,
        status,
        no_lookahead_safe,
        reason_codes: stable_reason_codes(&reason_codes),
    }
}

fn timestamp_exists(series: &CandleSeries, timestamp_ms: u64, tolerance_ms: u64) -> bool {
    series.candles.iter().any(|candle| {
        let distance =
            candle.timestamp_ms.max(timestamp_ms) - candle.timestamp_ms.min(timestamp_ms);
        distance <= tolerance_ms
    })
}

fn window_has_gap(series: &CandleSeries, start_index: usize, end_index: usize) -> bool {
    let expected_step = series
        .candles
        .windows(2)
        .filter_map(|window| {
            let step = window[1]
                .timestamp_ms
                .saturating_sub(window[0].timestamp_ms);
            (step > 0).then_some(step)
        })
        .min()
        .unwrap_or(0);
    if expected_step == 0 {
        return false;
    }
    series.candles[start_index..=end_index]
        .windows(2)
        .any(|window| {
            window[1]
                .timestamp_ms
                .saturating_sub(window[0].timestamp_ms)
                != expected_step
        })
}

fn determine_alignment_status(
    records: &[CandleAlignmentRecord],
    matched_count: usize,
    tolerance_match_count: usize,
    insufficient_future_bars_count: usize,
    no_lookahead_rejected_count: usize,
) -> CandleAlignmentOverallStatus {
    if records
        .iter()
        .any(|record| record.status == CandleAlignmentStatus::BadDataQuality)
    {
        CandleAlignmentOverallStatus::BadDataQuality
    } else if no_lookahead_rejected_count > 0 {
        CandleAlignmentOverallStatus::NeedBetterTimestampAlignment
    } else if insufficient_future_bars_count > 0 {
        CandleAlignmentOverallStatus::NeedLongerFutureWindows
    } else if matched_count == 0 {
        CandleAlignmentOverallStatus::NeedMoreCandleData
    } else if records.iter().any(|record| {
        matches!(
            record.status,
            CandleAlignmentStatus::MissingCandleSeries
                | CandleAlignmentStatus::MissingTimestamp
                | CandleAlignmentStatus::WrongSymbol
        )
    }) {
        CandleAlignmentOverallStatus::NeedMoreCandleData
    } else if tolerance_match_count > 0 {
        CandleAlignmentOverallStatus::NeedBetterTimestampAlignment
    } else if records.iter().all(|record| {
        record.status == CandleAlignmentStatus::MatchedWithTolerance
            || record.status == CandleAlignmentStatus::MatchedExact
    }) && records
        .iter()
        .all(|record| record.status == CandleAlignmentStatus::MatchedWithTolerance)
    {
        CandleAlignmentOverallStatus::DiagnosticOnly
    } else {
        CandleAlignmentOverallStatus::HealthyAlignment
    }
}

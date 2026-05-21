use serde::{Deserialize, Serialize};

use crate::backtest::Timeframe;
use crate::core::ReasonCode;

use super::validation::ValidationStats;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DataQualitySeverity {
    Good,
    Warning,
    Bad,
    Unusable,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DataQualityReport {
    pub symbol: String,
    pub timeframe: Timeframe,
    pub row_count: usize,
    pub valid_row_count: usize,
    pub invalid_row_count: usize,
    pub dropped_row_count: usize,
    pub repaired_row_count: usize,
    pub duplicate_timestamp_count: usize,
    pub out_of_order_count: usize,
    pub gap_count: usize,
    pub max_gap_ms: u64,
    pub gap_ratio: f64,
    pub non_positive_price_count: usize,
    pub negative_volume_count: usize,
    pub ohlc_invariant_violation_count: usize,
    pub missing_bid_ask_count: usize,
    pub extreme_spread_count: usize,
    pub data_quality_score: f64,
    pub severity: DataQualitySeverity,
    pub reason_codes: Vec<ReasonCode>,
}

pub fn build_data_quality_report(
    symbol: impl Into<String>,
    timeframe: Timeframe,
    stats: &ValidationStats,
) -> DataQualityReport {
    let row_count = stats.input_row_count;
    let gap_ratio = if row_count > 1 {
        stats.gap_count as f64 / (row_count - 1) as f64
    } else {
        0.0
    };
    let invalid_ratio = if row_count > 0 {
        stats.invalid_row_count as f64 / row_count as f64
    } else {
        1.0
    };
    let repair_ratio = if row_count > 0 {
        stats.repaired_row_count as f64 / row_count as f64
    } else {
        0.0
    };
    let score = (1.0
        - invalid_ratio * 0.55
        - gap_ratio * 0.18
        - repair_ratio * 0.12
        - ratio(stats.extreme_spread_count, row_count) * 0.08
        - ratio(stats.missing_bid_ask_count, row_count) * 0.05
        - ratio(stats.negative_volume_count, row_count) * 0.15
        - ratio(stats.non_positive_price_count, row_count) * 0.35
        - ratio(stats.ohlc_invariant_violation_count, row_count) * 0.30)
        .clamp(0.0, 1.0);
    let severity =
        if row_count == 0 || stats.valid_row_count == 0 || invalid_ratio >= 0.40 || score < 0.45 {
            DataQualitySeverity::Unusable
        } else if invalid_ratio >= 0.20 || score < 0.70 {
            DataQualitySeverity::Bad
        } else if gap_ratio > 0.0 || stats.repaired_row_count > 0 || score < 0.90 {
            DataQualitySeverity::Warning
        } else {
            DataQualitySeverity::Good
        };

    let mut reason_codes = stats.reason_codes.clone();
    if stats.gap_count > 0 {
        reason_codes.push(ReasonCode::GapDetected);
    }
    if stats.duplicate_timestamp_count > 0 {
        reason_codes.push(ReasonCode::DuplicateTimestampDetected);
    }
    if stats.out_of_order_count > 0 {
        reason_codes.push(ReasonCode::OutOfOrderTimestampDetected);
    }
    if stats.repaired_row_count > 0 {
        reason_codes.push(ReasonCode::CsvSortedRepairApplied);
    }
    reason_codes.push(match severity {
        DataQualitySeverity::Good => ReasonCode::CsvLoaded,
        DataQualitySeverity::Warning => ReasonCode::DataQualityWarning,
        DataQualitySeverity::Bad => ReasonCode::DataQualityBad,
        DataQualitySeverity::Unusable => ReasonCode::DataQualityUnusable,
    });

    DataQualityReport {
        symbol: symbol.into(),
        timeframe,
        row_count,
        valid_row_count: stats.valid_row_count,
        invalid_row_count: stats.invalid_row_count,
        dropped_row_count: stats.dropped_row_count,
        repaired_row_count: stats.repaired_row_count,
        duplicate_timestamp_count: stats.duplicate_timestamp_count,
        out_of_order_count: stats.out_of_order_count,
        gap_count: stats.gap_count,
        max_gap_ms: stats.max_gap_ms,
        gap_ratio,
        non_positive_price_count: stats.non_positive_price_count,
        negative_volume_count: stats.negative_volume_count,
        ohlc_invariant_violation_count: stats.ohlc_invariant_violation_count,
        missing_bid_ask_count: stats.missing_bid_ask_count,
        extreme_spread_count: stats.extreme_spread_count,
        data_quality_score: score,
        severity,
        reason_codes,
    }
}

fn ratio(count: usize, total: usize) -> f64 {
    if total == 0 {
        0.0
    } else {
        count as f64 / total as f64
    }
}

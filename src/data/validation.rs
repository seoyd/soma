use serde::{Deserialize, Serialize};

use crate::backtest::Candle;
use crate::core::ReasonCode;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CandleParseError {
    MissingColumn,
    InvalidNumber,
    InvalidTimestamp,
    NegativeVolume,
    NonPositivePrice,
    OhlcInvariantViolation,
    DuplicateTimestamp,
    OutOfOrderTimestamp,
    UnsupportedFormat,
    TooManyInvalidRows,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CandleParseIssue {
    pub row_number: Option<usize>,
    pub column: Option<String>,
    pub value: Option<String>,
    pub error: CandleParseError,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DataValidationConfig {
    pub strict: bool,
    pub allow_sort_repair: bool,
    pub allow_duplicate_drop: bool,
    pub allow_gap: bool,
    pub max_gap_count: usize,
    pub max_gap_ratio: f64,
    pub max_invalid_ratio: f64,
    pub expected_step_ms: Option<u64>,
    pub reason_codes: Vec<ReasonCode>,
}

impl Default for DataValidationConfig {
    fn default() -> Self {
        Self {
            strict: true,
            allow_sort_repair: false,
            allow_duplicate_drop: false,
            allow_gap: true,
            max_gap_count: usize::MAX,
            max_gap_ratio: 1.0,
            max_invalid_ratio: 1.0,
            expected_step_ms: None,
            reason_codes: vec![ReasonCode::DeterministicPath],
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ValidationStats {
    pub input_row_count: usize,
    pub valid_row_count: usize,
    pub invalid_row_count: usize,
    pub dropped_row_count: usize,
    pub repaired_row_count: usize,
    pub duplicate_timestamp_count: usize,
    pub out_of_order_count: usize,
    pub gap_count: usize,
    pub max_gap_ms: u64,
    pub non_positive_price_count: usize,
    pub negative_volume_count: usize,
    pub ohlc_invariant_violation_count: usize,
    pub missing_bid_ask_count: usize,
    pub extreme_spread_count: usize,
    pub reason_codes: Vec<ReasonCode>,
}

impl ValidationStats {
    pub fn observe_valid_candle(&mut self, candle: &Candle) {
        self.valid_row_count += 1;
        if candle.bid.is_none() || candle.ask.is_none() {
            self.missing_bid_ask_count += 1;
        }
        let spread_bps = candle.spread_bps.unwrap_or(0.0);
        if spread_bps > 25.0 {
            self.extreme_spread_count += 1;
        }
    }

    pub fn observe_issue(&mut self, issue: &CandleParseIssue) {
        self.invalid_row_count += 1;
        match issue.error {
            CandleParseError::NegativeVolume => self.negative_volume_count += 1,
            CandleParseError::NonPositivePrice => self.non_positive_price_count += 1,
            CandleParseError::OhlcInvariantViolation => {
                self.ohlc_invariant_violation_count += 1;
            }
            CandleParseError::DuplicateTimestamp => self.duplicate_timestamp_count += 1,
            CandleParseError::OutOfOrderTimestamp => self.out_of_order_count += 1,
            _ => {}
        }
    }
}

pub fn validate_candle(candle: &Candle, row_number: usize) -> Vec<CandleParseIssue> {
    let mut issues = Vec::new();
    if candle.open <= 0.0 || candle.high <= 0.0 || candle.low <= 0.0 || candle.close <= 0.0 {
        issues.push(CandleParseIssue {
            row_number: Some(row_number),
            column: None,
            value: None,
            error: CandleParseError::NonPositivePrice,
            reason_codes: vec![ReasonCode::NonPositivePrice],
        });
    }
    if candle.volume < 0.0 {
        issues.push(CandleParseIssue {
            row_number: Some(row_number),
            column: Some("volume".to_string()),
            value: Some(candle.volume.to_string()),
            error: CandleParseError::NegativeVolume,
            reason_codes: vec![ReasonCode::NegativeVolumeDetected],
        });
    }
    let valid_ohlc = candle.high >= candle.open
        && candle.high >= candle.close
        && candle.low <= candle.open
        && candle.low <= candle.close
        && candle.high >= candle.low;
    if !valid_ohlc {
        issues.push(CandleParseIssue {
            row_number: Some(row_number),
            column: None,
            value: None,
            error: CandleParseError::OhlcInvariantViolation,
            reason_codes: vec![ReasonCode::OhlcInvariantViolationDetected],
        });
    }
    if let (Some(bid), Some(ask)) = (candle.bid, candle.ask)
        && bid > ask
    {
        issues.push(CandleParseIssue {
            row_number: Some(row_number),
            column: Some("bid/ask".to_string()),
            value: Some(format!("{bid}/{ask}")),
            error: CandleParseError::OhlcInvariantViolation,
            reason_codes: vec![ReasonCode::OhlcInvariantViolationDetected],
        });
    }
    if candle.spread_bps.is_some_and(|spread| spread < 0.0) {
        issues.push(CandleParseIssue {
            row_number: Some(row_number),
            column: Some("spread_bps".to_string()),
            value: candle.spread_bps.map(|value| value.to_string()),
            error: CandleParseError::OhlcInvariantViolation,
            reason_codes: vec![ReasonCode::OhlcInvariantViolationDetected],
        });
    }
    issues
}

pub fn detect_temporal_issues(
    candles: &[Candle],
    expected_step_ms: Option<u64>,
) -> (usize, usize, usize, u64) {
    let mut duplicates = 0usize;
    let mut out_of_order = 0usize;
    let mut gaps = 0usize;
    let mut max_gap_ms = 0u64;

    for pair in candles.windows(2) {
        let previous = &pair[0];
        let current = &pair[1];
        if current.timestamp_ms == previous.timestamp_ms {
            duplicates += 1;
        } else if current.timestamp_ms < previous.timestamp_ms {
            out_of_order += 1;
        } else if let Some(step) = expected_step_ms {
            let delta = current.timestamp_ms - previous.timestamp_ms;
            if delta > step {
                gaps += 1;
                max_gap_ms = max_gap_ms.max(delta - step);
            }
        }
    }

    (duplicates, out_of_order, gaps, max_gap_ms)
}

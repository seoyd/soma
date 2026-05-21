use serde::{Deserialize, Serialize};

use crate::backtest::{CandleSeries, TripleBarrierConfig, evaluate_triple_barrier};
use crate::core::ReasonCode;
use crate::feature::FeatureEngine;

use super::walk_forward::WalkForwardFold;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LeakageReport {
    pub has_leakage: bool,
    pub warnings: Vec<ReasonCode>,
    pub unsafe_rows_count: usize,
    pub checked_rows_count: usize,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LeakageGuard;

impl LeakageGuard {
    pub fn analyze_fold(fold: &WalkForwardFold, horizon_bars: usize) -> LeakageReport {
        let mut warnings = Vec::new();
        let mut reason_codes = Vec::new();
        let mut has_leakage = false;

        if fold.train_end_index >= fold.test_start_index {
            warnings.push(ReasonCode::FoldOverlapDetected);
            reason_codes.push(ReasonCode::LeakageDetected);
            has_leakage = true;
        }

        if let (Some(validation_start), Some(validation_end)) =
            (fold.validation_start_index, fold.validation_end_index)
        {
            if validation_start <= fold.train_end_index || validation_end >= fold.test_start_index {
                warnings.push(ReasonCode::FoldOverlapDetected);
                reason_codes.push(ReasonCode::LeakageDetected);
                has_leakage = true;
            }
        }

        if let (Some(embargo_start), Some(embargo_end)) =
            (fold.embargo_start_index, fold.embargo_end_index)
        {
            if embargo_start > embargo_end || embargo_end >= fold.test_start_index {
                warnings.push(ReasonCode::FoldOverlapDetected);
                reason_codes.push(ReasonCode::LeakageDetected);
                has_leakage = true;
            }
        }

        let train_unsafe = unsafe_count_for_range(
            fold.train_start_index,
            fold.train_end_index,
            fold.train_end_index,
            horizon_bars,
        );
        let validation_unsafe = match (fold.validation_start_index, fold.validation_end_index) {
            (Some(start), Some(end)) => unsafe_count_for_range(start, end, end, horizon_bars),
            _ => 0,
        };
        let test_unsafe = unsafe_count_for_range(
            fold.test_start_index,
            fold.test_end_index,
            fold.test_end_index,
            horizon_bars,
        );
        let unsafe_rows_count = train_unsafe + validation_unsafe + test_unsafe;
        if unsafe_rows_count > 0 {
            warnings.push(ReasonCode::UnsafeLabelBoundary);
        }

        let checked_rows_count =
            checked_count_for_range(fold.train_start_index, fold.train_end_index)
                + count_optional_range(fold.validation_start_index, fold.validation_end_index)
                + checked_count_for_range(fold.test_start_index, fold.test_end_index);

        if !warnings.is_empty() && !reason_codes.contains(&ReasonCode::UnsafeLabelBoundary) {
            reason_codes.extend(warnings.iter().cloned());
        }

        LeakageReport {
            has_leakage,
            warnings,
            unsafe_rows_count,
            checked_rows_count,
            reason_codes,
        }
    }

    pub fn feature_stable_at(
        engine: &FeatureEngine,
        before: &CandleSeries,
        after: &CandleSeries,
        index: usize,
    ) -> bool {
        engine.build_at(before, index) == engine.build_at(after, index)
    }

    pub fn label_changes_only_in_label_stage(
        before: &CandleSeries,
        after: &CandleSeries,
        index: usize,
        entry_price: f64,
        config: TripleBarrierConfig,
    ) -> bool {
        evaluate_triple_barrier(before, index, entry_price, config)
            != evaluate_triple_barrier(after, index, entry_price, config)
    }
}

pub fn row_is_unsafe(index: usize, split_end_index: usize, horizon_bars: usize) -> bool {
    index.saturating_add(horizon_bars) > split_end_index
}

fn unsafe_count_for_range(
    start_index: usize,
    end_index: usize,
    split_end_index: usize,
    horizon_bars: usize,
) -> usize {
    if start_index > end_index {
        return 0;
    }
    let safe_end = split_end_index.saturating_sub(horizon_bars);
    if safe_end < start_index {
        end_index - start_index + 1
    } else if safe_end >= end_index {
        0
    } else {
        end_index - safe_end
    }
}

fn checked_count_for_range(start_index: usize, end_index: usize) -> usize {
    if start_index > end_index {
        0
    } else {
        end_index - start_index + 1
    }
}

fn count_optional_range(start_index: Option<usize>, end_index: Option<usize>) -> usize {
    match (start_index, end_index) {
        (Some(start), Some(end)) if start <= end => end - start + 1,
        _ => 0,
    }
}

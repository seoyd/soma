use std::collections::BTreeMap;
use std::fs;

use serde::{Deserialize, Serialize};

use crate::core::ReasonCode;

use super::source_inventory::SourceDatasetRecord;
use super::source_overlap::{SourceOverlapKey, SourceOverlapReport};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SourceMismatchSeverity {
    None,
    Low,
    Medium,
    High,
    NotComparable,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SourceMismatchReport {
    pub overlap_key: SourceOverlapKey,
    pub row_count_delta: i64,
    pub timestamp_mismatch_count: usize,
    pub missing_row_count: usize,
    pub price_drift_bps_avg: f64,
    pub price_drift_bps_max: f64,
    pub volume_delta_ratio_avg: f64,
    pub adjusted_policy_mismatch: bool,
    pub gap_mismatch_count: usize,
    pub data_quality_delta: f64,
    pub severity: SourceMismatchSeverity,
    pub warnings: Vec<String>,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SourceMismatchAggregate {
    pub reports: Vec<SourceMismatchReport>,
    pub high_severity_count: usize,
    pub not_comparable_count: usize,
    pub avg_price_drift_bps: f64,
    pub max_price_drift_bps: f64,
    pub most_common_warnings: Vec<String>,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq)]
struct CanonicalRow {
    timestamp_ms: u64,
    close: f64,
    volume: f64,
}

pub fn build_source_mismatch_report(
    overlap_key: SourceOverlapKey,
    official: &SourceDatasetRecord,
    yfinance: &SourceDatasetRecord,
    max_allowed_source_price_drift_bps: f64,
) -> Result<SourceMismatchReport, String> {
    let official_path = official
        .canonical_csv_path
        .as_deref()
        .ok_or_else(|| "official canonical csv path is missing".to_string())?;
    let yfinance_path = yfinance
        .canonical_csv_path
        .as_deref()
        .ok_or_else(|| "yfinance canonical csv path is missing".to_string())?;
    let official_rows = load_canonical_rows(official_path)?;
    let yfinance_rows = load_canonical_rows(yfinance_path)?;
    let adjusted_policy_mismatch = official.adjusted_price_policy != yfinance.adjusted_price_policy
        && official.adjusted_price_policy.is_some()
        && yfinance.adjusted_price_policy.is_some();

    let official_map = official_rows
        .iter()
        .map(|row| (row.timestamp_ms, row))
        .collect::<BTreeMap<_, _>>();
    let yfinance_map = yfinance_rows
        .iter()
        .map(|row| (row.timestamp_ms, row))
        .collect::<BTreeMap<_, _>>();
    let mut common = Vec::new();
    let mut timestamp_mismatch_count = 0usize;
    for timestamp in official_map.keys() {
        if let Some(y_row) = yfinance_map.get(timestamp) {
            common.push((official_map[timestamp].clone(), (*y_row).clone()));
        } else {
            timestamp_mismatch_count += 1;
        }
    }
    for timestamp in yfinance_map.keys() {
        if !official_map.contains_key(timestamp) {
            timestamp_mismatch_count += 1;
        }
    }

    let missing_row_count = timestamp_mismatch_count;
    let mut price_drifts = Vec::new();
    let mut volume_ratios = Vec::new();
    for (official_row, yfinance_row) in common {
        let price_base = official_row.close.abs().max(1e-9);
        price_drifts
            .push(((official_row.close - yfinance_row.close).abs() / price_base) * 10_000.0);
        let volume_base = official_row
            .volume
            .abs()
            .max(yfinance_row.volume.abs())
            .max(1.0);
        volume_ratios.push((official_row.volume - yfinance_row.volume).abs() / volume_base);
    }

    let gap_mismatch_count = count_gaps(&official_rows).abs_diff(count_gaps(&yfinance_rows));
    let data_quality_delta = (official.data_quality_score.unwrap_or(0.0)
        - yfinance.data_quality_score.unwrap_or(0.0))
    .abs();
    let price_drift_bps_avg = average(&price_drifts);
    let price_drift_bps_max = max_value(&price_drifts);
    let volume_delta_ratio_avg = average(&volume_ratios);

    let severity = if adjusted_policy_mismatch || price_drifts.is_empty() {
        SourceMismatchSeverity::NotComparable
    } else if price_drift_bps_max > max_allowed_source_price_drift_bps
        || timestamp_mismatch_count > 3
        || gap_mismatch_count > 2
    {
        SourceMismatchSeverity::High
    } else if price_drift_bps_max > max_allowed_source_price_drift_bps * 0.5
        || volume_delta_ratio_avg > 0.3
        || data_quality_delta > 0.15
    {
        SourceMismatchSeverity::Medium
    } else if price_drift_bps_avg > 0.0 || volume_delta_ratio_avg > 0.0 {
        SourceMismatchSeverity::Low
    } else {
        SourceMismatchSeverity::None
    };

    let mut warnings = Vec::new();
    if adjusted_policy_mismatch {
        warnings.push("adjusted price policy mismatch".to_string());
    }
    if timestamp_mismatch_count > 0 {
        warnings.push("timestamp mismatch detected".to_string());
    }
    if gap_mismatch_count > 0 {
        warnings.push("gap mismatch detected".to_string());
    }
    if price_drift_bps_max > max_allowed_source_price_drift_bps {
        warnings.push("price drift exceeds configured threshold".to_string());
    }

    Ok(SourceMismatchReport {
        overlap_key,
        row_count_delta: official_rows.len() as i64 - yfinance_rows.len() as i64,
        timestamp_mismatch_count,
        missing_row_count,
        price_drift_bps_avg,
        price_drift_bps_max,
        volume_delta_ratio_avg,
        adjusted_policy_mismatch,
        gap_mismatch_count,
        data_quality_delta,
        severity,
        warnings,
        reason_codes: vec![ReasonCode::SourceMismatchBuilt],
    })
}

pub fn build_source_mismatch_aggregate(
    overlap_report: &SourceOverlapReport,
    official: &[SourceDatasetRecord],
    yfinance: &[SourceDatasetRecord],
    max_allowed_source_price_drift_bps: f64,
) -> Result<SourceMismatchAggregate, String> {
    let mut reports = Vec::new();
    for key in &overlap_report.overlap_keys {
        let Some(official_record) = official.iter().find(|record| {
            record.normalized_symbol == key.normalized_symbol
                && record.timeframe_label == key.timeframe_label
        }) else {
            continue;
        };
        let Some(yfinance_record) = yfinance.iter().find(|record| {
            record.normalized_symbol == key.normalized_symbol
                && record.timeframe_label == key.timeframe_label
        }) else {
            continue;
        };
        reports.push(build_source_mismatch_report(
            key.clone(),
            official_record,
            yfinance_record,
            max_allowed_source_price_drift_bps,
        )?);
    }

    let high_severity_count = reports
        .iter()
        .filter(|report| report.severity == SourceMismatchSeverity::High)
        .count();
    let not_comparable_count = reports
        .iter()
        .filter(|report| report.severity == SourceMismatchSeverity::NotComparable)
        .count();
    let avg_price_drift_bps = average(
        &reports
            .iter()
            .map(|report| report.price_drift_bps_avg)
            .collect::<Vec<_>>(),
    );
    let max_price_drift_bps = max_value(
        &reports
            .iter()
            .map(|report| report.price_drift_bps_max)
            .collect::<Vec<_>>(),
    );
    let warning_counts = reports
        .iter()
        .flat_map(|report| report.warnings.iter().cloned())
        .fold(BTreeMap::new(), |mut acc, warning| {
            *acc.entry(warning).or_insert(0usize) += 1;
            acc
        });
    let most_common_warnings = warning_counts.into_iter().collect::<Vec<_>>();
    let most_common_warnings = most_common_warnings
        .into_iter()
        .map(|(warning, count)| format!("{warning}:{count}"))
        .collect::<Vec<_>>();

    Ok(SourceMismatchAggregate {
        reports,
        high_severity_count,
        not_comparable_count,
        avg_price_drift_bps,
        max_price_drift_bps,
        most_common_warnings,
        reason_codes: vec![ReasonCode::SourceMismatchBuilt],
    })
}

fn load_canonical_rows(path: &str) -> Result<Vec<CanonicalRow>, String> {
    let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
    let mut rows = Vec::new();
    for (index, line) in text.lines().enumerate() {
        if index == 0 || line.trim().is_empty() {
            continue;
        }
        let parts = line.split(',').collect::<Vec<_>>();
        if parts.len() < 6 {
            return Err(format!("invalid canonical row in {path}"));
        }
        rows.push(CanonicalRow {
            timestamp_ms: parts[0].parse::<u64>().map_err(|err| err.to_string())?,
            close: parts[4].parse::<f64>().map_err(|err| err.to_string())?,
            volume: parts[5].parse::<f64>().map_err(|err| err.to_string())?,
        });
    }
    Ok(rows)
}

fn count_gaps(rows: &[CanonicalRow]) -> usize {
    rows.windows(2)
        .filter(|pair| pair[1].timestamp_ms.saturating_sub(pair[0].timestamp_ms) > 86_400_000)
        .count()
}

fn average(values: &[f64]) -> f64 {
    if values.is_empty() {
        0.0
    } else {
        values.iter().sum::<f64>() / values.len() as f64
    }
}

fn max_value(values: &[f64]) -> f64 {
    values.iter().copied().fold(0.0, f64::max)
}

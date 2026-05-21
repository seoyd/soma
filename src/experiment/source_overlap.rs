use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::core::ReasonCode;

use super::source_inventory::SourceDatasetRecord;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SourceOverlapKey {
    pub normalized_symbol: String,
    pub timeframe_label: String,
    #[serde(default)]
    pub date_range_bucket: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SourceOverlapReport {
    pub overlap_keys: Vec<SourceOverlapKey>,
    pub official_only_keys: Vec<SourceOverlapKey>,
    pub yfinance_only_keys: Vec<SourceOverlapKey>,
    pub missing_official_for_yfinance: Vec<SourceOverlapKey>,
    pub missing_yfinance_for_official: Vec<SourceOverlapKey>,
    pub overlap_count: usize,
    pub comparable: bool,
    pub reason_codes: Vec<ReasonCode>,
}

pub fn build_source_overlap_report(
    official: &[SourceDatasetRecord],
    yfinance: &[SourceDatasetRecord],
) -> SourceOverlapReport {
    let official_keys = official
        .iter()
        .map(overlap_key)
        .collect::<BTreeSet<SourceOverlapKey>>();
    let yfinance_keys = yfinance
        .iter()
        .map(overlap_key)
        .collect::<BTreeSet<SourceOverlapKey>>();

    let overlap_keys = official_keys
        .intersection(&yfinance_keys)
        .cloned()
        .collect::<Vec<_>>();
    let official_only_keys = official_keys
        .difference(&yfinance_keys)
        .cloned()
        .collect::<Vec<_>>();
    let yfinance_only_keys = yfinance_keys
        .difference(&official_keys)
        .cloned()
        .collect::<Vec<_>>();
    let adjusted_policy_mismatch = overlap_keys.iter().any(|key| {
        let official_record = official.iter().find(|record| {
            record.normalized_symbol == key.normalized_symbol
                && record.timeframe_label == key.timeframe_label
        });
        let yfinance_record = yfinance.iter().find(|record| {
            record.normalized_symbol == key.normalized_symbol
                && record.timeframe_label == key.timeframe_label
        });
        official_record
            .zip(yfinance_record)
            .is_some_and(|(left, right)| {
                left.adjusted_price_policy != right.adjusted_price_policy
                    && left.adjusted_price_policy.is_some()
                    && right.adjusted_price_policy.is_some()
            })
    });
    let comparable = !overlap_keys.is_empty() && !adjusted_policy_mismatch;

    SourceOverlapReport {
        overlap_keys: overlap_keys.clone(),
        official_only_keys: official_only_keys.clone(),
        yfinance_only_keys: yfinance_only_keys.clone(),
        missing_official_for_yfinance: yfinance_only_keys,
        missing_yfinance_for_official: official_only_keys,
        overlap_count: overlap_keys.len(),
        comparable,
        reason_codes: vec![ReasonCode::SourceOverlapBuilt],
    }
}

fn overlap_key(record: &SourceDatasetRecord) -> SourceOverlapKey {
    SourceOverlapKey {
        normalized_symbol: record.normalized_symbol.clone(),
        timeframe_label: record.timeframe_label.clone(),
        date_range_bucket: None,
    }
}

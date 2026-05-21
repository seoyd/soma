use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_reason_codes};
use crate::data::{ProviderKind, ProviderMarket};

use super::comparable_committee_evidence::{
    ComparableCommitteeEvidenceRow, ComparableEvidenceSourceClass,
};
use super::official_candle_coverage_pack::{normalize_symbol, normalize_timeframe_label};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TimestampPolicyKind {
    ExactEpochMs,
    DailyMidnightUtc,
    DailySessionOpen,
    DailySessionClose,
    ProviderSpecific,
    #[default]
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RawMatchKey {
    pub market: ProviderMarket,
    #[serde(default)]
    pub venue: Option<String>,
    #[serde(default)]
    pub provider_kind: Option<ProviderKind>,
    #[serde(default)]
    pub provider_symbol: Option<String>,
    pub raw_symbol: String,
    pub timeframe: String,
    pub horizon_bars: usize,
    pub timestamp_ms: u64,
    pub source_class: ComparableEvidenceSourceClass,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NormalizedMatchKey {
    pub market: ProviderMarket,
    #[serde(default)]
    pub venue: Option<String>,
    #[serde(default)]
    pub provider_kind: Option<ProviderKind>,
    #[serde(default)]
    pub provider_symbol: Option<String>,
    pub raw_symbol: String,
    pub normalized_symbol: String,
    pub timeframe: String,
    pub normalized_timeframe: String,
    pub horizon_bars: usize,
    pub timestamp_ms: u64,
    pub timestamp_policy: TimestampPolicyKind,
    #[serde(default)]
    pub adjusted_price_policy: Option<String>,
    pub source_class: ComparableEvidenceSourceClass,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum MatchKeyNormalizationStatus {
    Normalized,
    MissingSymbol,
    MissingMarket,
    MissingTimeframe,
    MissingTimestamp,
    AliasMissing,
    TimeframeAliasMissing,
    TimestampPolicyMissing,
    SourceIneligible,
    #[default]
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MatchKeyNormalizationReport {
    pub row_id: String,
    pub raw_key: RawMatchKey,
    pub normalized_key: NormalizedMatchKey,
    pub alias_applied: bool,
    pub timeframe_alias_applied: bool,
    pub timestamp_policy_applied: bool,
    pub normalization_status: MatchKeyNormalizationStatus,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MatchKeyNormalizationAggregate {
    pub reports: Vec<MatchKeyNormalizationReport>,
    pub normalized_count: usize,
    pub failed_count: usize,
    pub alias_applied_count: usize,
    pub timeframe_alias_applied_count: usize,
    pub timestamp_policy_applied_count: usize,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MatchKeyNormalizationOptions {
    pub allow_explicit_symbol_alias: bool,
    pub allow_explicit_timeframe_alias: bool,
    pub allow_explicit_timestamp_policy_map: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SymbolAliasMap {
    #[serde(default)]
    pub aliases: Vec<SymbolAliasEntry>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SymbolAliasEntry {
    pub raw_symbol: String,
    pub normalized_symbol: String,
    #[serde(default)]
    pub provider_symbol: Option<String>,
    #[serde(default)]
    pub market: Option<ProviderMarket>,
    #[serde(default)]
    pub venue: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TimeframeAliasMap {
    #[serde(default)]
    pub aliases: Vec<TimeframeAliasEntry>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TimeframeAliasEntry {
    pub raw_timeframe: String,
    pub normalized_timeframe: String,
    #[serde(default)]
    pub market: Option<ProviderMarket>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TimestampPolicyMap {
    #[serde(default)]
    pub policies: Vec<TimestampPolicyEntry>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TimestampPolicyEntry {
    #[serde(default)]
    pub row_id: Option<String>,
    #[serde(default)]
    pub raw_symbol: Option<String>,
    #[serde(default)]
    pub raw_timeframe: Option<String>,
    pub timestamp_policy: TimestampPolicyKind,
    #[serde(default)]
    pub session_policy: Option<String>,
}

pub fn load_symbol_alias_map(path: &str) -> Result<SymbolAliasMap, String> {
    load_local_toml_map(path)
}

pub fn load_timeframe_alias_map(path: &str) -> Result<TimeframeAliasMap, String> {
    load_local_toml_map(path)
}

pub fn load_timestamp_policy_map(path: &str) -> Result<TimestampPolicyMap, String> {
    load_local_toml_map(path)
}

pub fn build_match_key_normalization_aggregate(
    rows: &[ComparableCommitteeEvidenceRow],
    options: &MatchKeyNormalizationOptions,
    symbol_alias_map: Option<&SymbolAliasMap>,
    timeframe_alias_map: Option<&TimeframeAliasMap>,
    timestamp_policy_map: Option<&TimestampPolicyMap>,
) -> MatchKeyNormalizationAggregate {
    let mut reports = rows
        .iter()
        .map(|row| {
            normalize_row_match_key(
                row,
                options,
                symbol_alias_map,
                timeframe_alias_map,
                timestamp_policy_map,
            )
        })
        .collect::<Vec<_>>();
    reports.sort_by(|left, right| left.row_id.cmp(&right.row_id));
    let normalized_count = reports
        .iter()
        .filter(|report| report.normalization_status == MatchKeyNormalizationStatus::Normalized)
        .count();
    let failed_count = reports.len().saturating_sub(normalized_count);
    let alias_applied_count = reports.iter().filter(|report| report.alias_applied).count();
    let timeframe_alias_applied_count = reports
        .iter()
        .filter(|report| report.timeframe_alias_applied)
        .count();
    let timestamp_policy_applied_count = reports
        .iter()
        .filter(|report| report.timestamp_policy_applied)
        .count();
    MatchKeyNormalizationAggregate {
        reports,
        normalized_count,
        failed_count,
        alias_applied_count,
        timeframe_alias_applied_count,
        timestamp_policy_applied_count,
        reason_codes: stable_reason_codes(&[
            ReasonCode::DeterministicPath,
            ReasonCode::SymbolNormalized,
        ]),
    }
}

pub fn normalize_row_match_key(
    row: &ComparableCommitteeEvidenceRow,
    options: &MatchKeyNormalizationOptions,
    symbol_alias_map: Option<&SymbolAliasMap>,
    timeframe_alias_map: Option<&TimeframeAliasMap>,
    timestamp_policy_map: Option<&TimestampPolicyMap>,
) -> MatchKeyNormalizationReport {
    let mut reason_codes = row.reason_codes.clone();
    let raw_key = RawMatchKey {
        market: row.market,
        venue: None,
        provider_kind: None,
        provider_symbol: Some(row.symbol.clone()),
        raw_symbol: row.symbol.clone(),
        timeframe: row.timeframe.clone(),
        horizon_bars: row.horizon_bars,
        timestamp_ms: row.timestamp_ms,
        source_class: row.source_class,
        reason_codes: row.reason_codes.clone(),
    };

    let alias_entry = symbol_alias_map.and_then(|map| {
        map.aliases.iter().find(|entry| {
            entry.raw_symbol == row.symbol && entry.market.is_none_or(|market| market == row.market)
        })
    });
    let timeframe_entry = timeframe_alias_map.and_then(|map| {
        map.aliases.iter().find(|entry| {
            entry.raw_timeframe.eq_ignore_ascii_case(&row.timeframe)
                && entry.market.is_none_or(|market| market == row.market)
        })
    });
    let timestamp_entry = timestamp_policy_map.and_then(|map| {
        map.policies.iter().find(|entry| {
            entry.row_id.as_deref() == Some(row.row_id.as_str())
                || (entry.raw_symbol.as_deref() == Some(row.symbol.as_str())
                    && entry.raw_timeframe.as_deref() == Some(row.timeframe.as_str()))
        })
    });

    let alias_applied = options.allow_explicit_symbol_alias && alias_entry.is_some();
    let timeframe_alias_applied =
        options.allow_explicit_timeframe_alias && timeframe_entry.is_some();
    let timestamp_policy_applied =
        options.allow_explicit_timestamp_policy_map && timestamp_entry.is_some();

    let normalized_symbol = if alias_applied {
        reason_codes.push(ReasonCode::SymbolNormalized);
        alias_entry
            .map(|entry| entry.normalized_symbol.clone())
            .unwrap_or_else(|| normalize_symbol(&row.symbol))
    } else {
        normalize_symbol(&row.symbol)
    };
    let normalized_timeframe = if timeframe_alias_applied {
        timeframe_entry
            .map(|entry| normalize_timeframe_label(&entry.normalized_timeframe))
            .unwrap_or_else(|| normalize_timeframe_label(&row.timeframe))
    } else {
        normalize_timeframe_label(&row.timeframe)
    };
    let timestamp_policy = if timestamp_policy_applied {
        timestamp_entry
            .map(|entry| entry.timestamp_policy)
            .unwrap_or_else(|| infer_timestamp_policy(row.timestamp_ms, &row.timeframe))
    } else {
        infer_timestamp_policy(row.timestamp_ms, &row.timeframe)
    };

    let normalization_status = if row.symbol.trim().is_empty() {
        MatchKeyNormalizationStatus::MissingSymbol
    } else if row.timeframe.trim().is_empty() {
        MatchKeyNormalizationStatus::MissingTimeframe
    } else if row.timestamp_ms == 0 {
        MatchKeyNormalizationStatus::MissingTimestamp
    } else if matches!(
        row.source_class,
        ComparableEvidenceSourceClass::ControlledDiagnostic
            | ComparableEvidenceSourceClass::YFinanceResearch
            | ComparableEvidenceSourceClass::FixtureArchitectureTest
            | ComparableEvidenceSourceClass::SyntheticTest
            | ComparableEvidenceSourceClass::Unknown
    ) {
        MatchKeyNormalizationStatus::SourceIneligible
    } else if options.allow_explicit_symbol_alias
        && symbol_alias_map.is_some()
        && alias_entry.is_none()
    {
        MatchKeyNormalizationStatus::AliasMissing
    } else if options.allow_explicit_timeframe_alias
        && timeframe_alias_map.is_some()
        && timeframe_entry.is_none()
        && normalize_timeframe_label(&row.timeframe) != row.timeframe.to_ascii_lowercase()
    {
        MatchKeyNormalizationStatus::TimeframeAliasMissing
    } else if options.allow_explicit_timestamp_policy_map
        && timestamp_policy_map.is_some()
        && timestamp_entry.is_none()
        && normalize_timeframe_label(&row.timeframe) == "1d"
    {
        MatchKeyNormalizationStatus::TimestampPolicyMissing
    } else {
        MatchKeyNormalizationStatus::Normalized
    };

    MatchKeyNormalizationReport {
        row_id: row.row_id.clone(),
        raw_key,
        normalized_key: NormalizedMatchKey {
            market: row.market,
            venue: alias_entry.and_then(|entry| entry.venue.clone()),
            provider_kind: None,
            provider_symbol: alias_entry
                .and_then(|entry| entry.provider_symbol.clone())
                .or_else(|| Some(row.symbol.clone())),
            raw_symbol: row.symbol.clone(),
            normalized_symbol,
            timeframe: row.timeframe.clone(),
            normalized_timeframe,
            horizon_bars: row.horizon_bars,
            timestamp_ms: row.timestamp_ms,
            timestamp_policy,
            adjusted_price_policy: None,
            source_class: row.source_class,
            reason_codes: stable_reason_codes(&reason_codes),
        },
        alias_applied,
        timeframe_alias_applied,
        timestamp_policy_applied,
        normalization_status,
        reason_codes: stable_reason_codes(&reason_codes),
    }
}

pub fn infer_timestamp_policy(timestamp_ms: u64, timeframe: &str) -> TimestampPolicyKind {
    if timestamp_ms == 0 {
        return TimestampPolicyKind::Unknown;
    }
    if normalize_timeframe_label(timeframe) == "1d" && timestamp_ms % 86_400_000 == 0 {
        TimestampPolicyKind::DailyMidnightUtc
    } else {
        TimestampPolicyKind::ExactEpochMs
    }
}

pub fn reports_by_row_id(
    aggregate: &MatchKeyNormalizationAggregate,
) -> BTreeMap<String, MatchKeyNormalizationReport> {
    aggregate
        .reports
        .iter()
        .cloned()
        .map(|report| (report.row_id.clone(), report))
        .collect()
}

impl MatchKeyNormalizationAggregate {
    pub fn to_text(&self) -> String {
        let mut lines = vec![
            format!("normalized_count={}", self.normalized_count),
            format!("failed_count={}", self.failed_count),
            format!("alias_applied_count={}", self.alias_applied_count),
            format!(
                "timeframe_alias_applied_count={}",
                self.timeframe_alias_applied_count
            ),
            format!(
                "timestamp_policy_applied_count={}",
                self.timestamp_policy_applied_count
            ),
        ];
        lines.extend(self.reports.iter().map(|report| {
            format!(
                "row_id={};raw_symbol={};normalized_symbol={};raw_timeframe={};normalized_timeframe={};timestamp_policy={:?};status={:?};alias_applied={};timeframe_alias_applied={};timestamp_policy_applied={}",
                report.row_id,
                report.raw_key.raw_symbol,
                report.normalized_key.normalized_symbol,
                report.raw_key.timeframe,
                report.normalized_key.normalized_timeframe,
                report.normalized_key.timestamp_policy,
                report.normalization_status,
                report.alias_applied,
                report.timeframe_alias_applied,
                report.timestamp_policy_applied,
            )
        }));
        lines.join("\n")
    }
}

pub fn normalized_symbols(aggregate: &MatchKeyNormalizationAggregate) -> BTreeSet<String> {
    aggregate
        .reports
        .iter()
        .map(|report| report.normalized_key.normalized_symbol.clone())
        .collect()
}

fn load_local_toml_map<T>(path: &str) -> Result<T, String>
where
    T: for<'de> Deserialize<'de>,
{
    if path.contains("://") {
        return Err("normalization map paths must be local".to_string());
    }
    let text = fs::read_to_string(Path::new(path)).map_err(|err| err.to_string())?;
    toml::from_str(&text).map_err(|err| err.to_string())
}

#[path = "support/sprint44_support.rs"]
mod sprint44_support;

use soma_zero::{
    ComparableEvidenceSourceClass, MatchKeyNormalizationOptions, MatchKeyNormalizationStatus,
    build_match_key_normalization_aggregate, load_symbol_alias_map, load_timeframe_alias_map,
    load_timestamp_policy_map,
};

#[test]
fn match_key_normalization_applies_explicit_local_alias_timeframe_and_timestamp_policy() {
    let row = sprint44_support::load_row("examples/sprint44_data/repairable_official_bundle.json");
    let aggregate = build_match_key_normalization_aggregate(
        &[row.clone()],
        &MatchKeyNormalizationOptions {
            allow_explicit_symbol_alias: true,
            allow_explicit_timeframe_alias: true,
            allow_explicit_timestamp_policy_map: true,
        },
        Some(
            &load_symbol_alias_map("examples/sprint44_data/symbol_alias_map.toml")
                .expect("symbol map"),
        ),
        Some(
            &load_timeframe_alias_map("examples/sprint44_data/timeframe_alias_map.toml")
                .expect("timeframe map"),
        ),
        Some(
            &load_timestamp_policy_map("examples/sprint44_data/timestamp_policy_map.toml")
                .expect("timestamp map"),
        ),
    );
    let report = &aggregate.reports[0];
    assert_eq!(report.raw_key.raw_symbol, "AAPL.OQ");
    assert_eq!(report.raw_key.provider_symbol.as_deref(), Some("AAPL.OQ"));
    assert_eq!(report.normalized_key.normalized_symbol, "AAPL");
    assert_eq!(
        report.normalized_key.provider_symbol.as_deref(),
        Some("AAPL")
    );
    assert_eq!(report.normalized_key.normalized_timeframe, "1d");
    assert!(report.alias_applied);
    assert!(report.timeframe_alias_applied);
    assert!(report.timestamp_policy_applied);
    assert_eq!(
        report.normalization_status,
        MatchKeyNormalizationStatus::Normalized
    );
}

#[test]
fn match_key_normalization_preserves_source_boundaries_for_yfinance_and_is_deterministic() {
    let mut row =
        sprint44_support::load_row("examples/sprint42_data/comparable_yfinance_tsla_bundle.json");
    row.source_class = ComparableEvidenceSourceClass::YFinanceResearch;
    let first = build_match_key_normalization_aggregate(
        &[row.clone()],
        &MatchKeyNormalizationOptions::default(),
        None,
        None,
        None,
    );
    let second = build_match_key_normalization_aggregate(
        &[row],
        &MatchKeyNormalizationOptions::default(),
        None,
        None,
        None,
    );
    assert_eq!(
        first.reports[0].normalization_status,
        MatchKeyNormalizationStatus::SourceIneligible
    );
    assert_eq!(first, second);
}

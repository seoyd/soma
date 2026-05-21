#[path = "support/sprint45_support.rs"]
mod sprint45_support;

use soma_zero::{
    OfficialCandleCoveragePack, OfficialCandleSeriesDescriptor, OfficialCandleSeriesSourceClass,
    OutcomeBackfillGapKind, OutcomeBackfillSuggestedAction, build_outcome_reference_backfill_plan,
};

#[test]
fn outcome_plan_marks_buildable_and_future_window_actions() {
    let mut buildable = sprint45_support::row("a");
    buildable.outcome_reference_available = false;
    let mut short = sprint45_support::row("b");
    short.outcome_reference_available = false;
    short.matched_candle_series_id = Some("short-series".to_string());
    let pack = OfficialCandleCoveragePack {
        pack_id: "pack".to_string(),
        descriptors: vec![
            OfficialCandleSeriesDescriptor {
                candle_series_id: "series-aapl-1d".to_string(),
                path: "a".to_string(),
                provider_kind: None,
                source_kind: soma_zero::EvidenceSourceKind::OfficialApiCollected,
                source_class: OfficialCandleSeriesSourceClass::OfficialNonCrypto,
                market: soma_zero::ProviderMarket::USEquity,
                venue: None,
                symbol: "AAPL".to_string(),
                normalized_symbol: "AAPL".to_string(),
                timeframe: "1d".to_string(),
                row_count: 32,
                timestamp_start_ms: 1,
                timestamp_end_ms: 64,
                has_duplicates: false,
                has_gaps: false,
                data_quality_score: None,
                provenance_available: true,
                preflight_ready: true,
                manifest_available: true,
                timestamp_policy: None,
                adjusted_price_policy: None,
                official_readiness_eligible: true,
                benchmark_eligible: true,
                diagnostic_only: false,
                storage_bytes: 1,
                reason_codes: vec![],
            },
            OfficialCandleSeriesDescriptor {
                row_count: 10,
                candle_series_id: "short-series".to_string(),
                path: "b".to_string(),
                provider_kind: None,
                source_kind: soma_zero::EvidenceSourceKind::OfficialApiCollected,
                source_class: OfficialCandleSeriesSourceClass::OfficialNonCrypto,
                market: soma_zero::ProviderMarket::USEquity,
                venue: None,
                symbol: "AAPL".to_string(),
                normalized_symbol: "AAPL".to_string(),
                timeframe: "1d".to_string(),
                timestamp_start_ms: 1,
                timestamp_end_ms: 10,
                has_duplicates: false,
                has_gaps: false,
                data_quality_score: None,
                provenance_available: true,
                preflight_ready: true,
                manifest_available: true,
                timestamp_policy: None,
                adjusted_price_policy: None,
                official_readiness_eligible: true,
                benchmark_eligible: true,
                diagnostic_only: false,
                storage_bytes: 1,
                reason_codes: vec![],
            },
        ],
        official_non_crypto_series: vec![],
        official_crypto_series: vec![],
        controlled_series: vec![],
        yfinance_series: vec![],
        fixture_series: vec![],
        unknown_series: vec![],
        total_rows: 42,
        total_symbols: 1,
        total_timeframes: 1,
        storage_bytes: 2,
        readiness_eligible_series_count: 2,
        benchmark_eligible_series_count: 2,
        warnings: vec![],
        reason_codes: vec![],
    };
    let plan = build_outcome_reference_backfill_plan("plan", &[buildable, short], None, &[pack]);
    assert_eq!(plan.buildable_count, 1);
    assert_eq!(
        plan.items[0].suggested_action,
        OutcomeBackfillSuggestedAction::BuildTripleBarrierOutcome
    );
    assert_eq!(
        plan.items[1].gap_kind,
        OutcomeBackfillGapKind::MissingFutureBars
    );
}

#[test]
fn outcome_plan_blocks_timestamp_horizon_and_no_lookahead_and_is_deterministic() {
    let mut timestamp = sprint45_support::row("ts");
    timestamp.outcome_reference_available = false;
    timestamp.candle_match_status = Some("TimestampMismatch".to_string());
    let mut horizon = sprint45_support::row("hz");
    horizon.outcome_reference_available = false;
    horizon.candle_match_status = Some("TimeframeMismatch".to_string());
    let mut unsafe_row = sprint45_support::row("unsafe");
    unsafe_row.outcome_reference_available = false;
    unsafe_row.no_lookahead_safe = false;
    let first = build_outcome_reference_backfill_plan(
        "plan",
        &[timestamp.clone(), horizon.clone(), unsafe_row.clone()],
        None,
        &[],
    );
    let second =
        build_outcome_reference_backfill_plan("plan", &[timestamp, horizon, unsafe_row], None, &[]);
    assert_eq!(first.to_text(), second.to_text());
    assert_eq!(
        first.items[0].gap_kind,
        OutcomeBackfillGapKind::HorizonMismatch
    );
    assert_eq!(
        first.items[1].gap_kind,
        OutcomeBackfillGapKind::TimestampMismatch
    );
    assert_eq!(
        first.items[2].gap_kind,
        OutcomeBackfillGapKind::NoLookaheadViolation
    );
}

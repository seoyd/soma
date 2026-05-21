#[path = "support/sprint44_support.rs"]
mod sprint44_support;

use soma_zero::{
    MatchKeyNormalizationOptions, RowCandleCandidateOptions, RowCandleCandidateStatus,
    build_match_key_normalization_aggregate, build_row_candle_candidate_report,
    load_symbol_alias_map, load_timeframe_alias_map, load_timestamp_policy_map,
};

fn report_for(
    bundle_path: &str,
    allow_symbol: bool,
    allow_timeframe: bool,
    allow_timestamp: bool,
) -> soma_zero::RowCandleCandidateReport {
    let row = sprint44_support::load_row(bundle_path);
    let pack = sprint44_support::load_pack("examples/soma_candle_pack_official_controlled.toml");
    let normalization = build_match_key_normalization_aggregate(
        &[row.clone()],
        &MatchKeyNormalizationOptions {
            allow_explicit_symbol_alias: allow_symbol,
            allow_explicit_timeframe_alias: allow_timeframe,
            allow_explicit_timestamp_policy_map: allow_timestamp,
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
    build_row_candle_candidate_report(
        &[row],
        &pack,
        &normalization,
        &RowCandleCandidateOptions {
            allow_session_daily_alignment: allow_timestamp,
            allow_timestamp_tolerance: allow_timestamp,
            ..RowCandleCandidateOptions::default()
        },
    )
}

#[test]
fn row_candle_candidate_reports_exact_match_and_repairable_mismatches() {
    let exact = report_for(
        "examples/sprint42_data/comparable_official_aapl_bundle.json",
        true,
        true,
        true,
    );
    assert_eq!(
        exact.candidates_by_row[0].status,
        RowCandleCandidateStatus::CandidateFound
    );
    assert_eq!(exact.official_ready_candidate_count, 1);

    let symbol = report_for(
        "examples/sprint44_data/repairable_official_bundle.json",
        false,
        false,
        false,
    );
    assert_eq!(
        symbol.candidates_by_row[0].status,
        RowCandleCandidateStatus::SymbolMismatch
    );

    let timeframe = report_for(
        "examples/sprint44_data/timeframe_mismatch_bundle.json",
        true,
        false,
        false,
    );
    assert_eq!(
        timeframe.candidates_by_row[0].status,
        RowCandleCandidateStatus::TimeframeMismatch
    );

    let timestamp = report_for(
        "examples/sprint44_data/timestamp_mismatch_bundle.json",
        true,
        true,
        false,
    );
    assert_eq!(
        timestamp.candidates_by_row[0].status,
        RowCandleCandidateStatus::TimestampOutsideRange
    );

    let future = report_for(
        "examples/sprint44_data/missing_future_window_bundle.json",
        true,
        true,
        true,
    );
    assert_eq!(
        future.candidates_by_row[0].status,
        RowCandleCandidateStatus::MissingFutureWindow
    );
}

#[test]
fn row_candle_candidate_handles_missing_provenance_preflight_and_multiple_candidates_deterministically()
 {
    let row =
        sprint44_support::load_row("examples/sprint42_data/comparable_official_aapl_bundle.json");
    let pack = sprint44_support::load_pack("examples/soma_candle_pack_official_controlled.toml");
    let normalization = build_match_key_normalization_aggregate(
        &[row.clone()],
        &MatchKeyNormalizationOptions::default(),
        None,
        None,
        None,
    );
    let first = build_row_candle_candidate_report(
        &[row.clone()],
        &pack,
        &normalization,
        &RowCandleCandidateOptions::default(),
    );
    let second = build_row_candle_candidate_report(
        &[row],
        &pack,
        &normalization,
        &RowCandleCandidateOptions::default(),
    );
    assert_eq!(
        first.candidates_by_row[0].selected_candle_series_id,
        second.candidates_by_row[0].selected_candle_series_id
    );
}

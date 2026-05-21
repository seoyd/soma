mod common;
#[path = "support/official_committee_support.rs"]
mod official_committee_support;

use soma_zero::{
    CommitteeCounterfactualBuildConfig, CommitteeCounterfactualBuilder, CounterfactualBuildStatus,
    load_local_candle_series_map,
};

#[test]
fn counterfactual_builder_builds_records_and_is_deterministic() {
    let (_, linked) = official_committee_support::build_controlled_linked_pack(
        "counterfactual-builder-built",
        true,
    );
    let candle_path = official_committee_support::write_candle_series(
        "counterfactual-builder-built",
        "AAPL",
        1_700_000_000_000,
        1.0,
    );
    let series =
        load_local_candle_series_map(&[candle_path.display().to_string()]).expect("series");
    let records = CommitteeCounterfactualBuilder::default().build_records(
        &linked.linked_rows[0],
        series.get("AAPL"),
        &CommitteeCounterfactualBuildConfig::default(),
    );
    let repeat = CommitteeCounterfactualBuilder::default().build_records(
        &linked.linked_rows[0],
        series.get("AAPL"),
        &CommitteeCounterfactualBuildConfig::default(),
    );
    assert_eq!(records, repeat);
    assert_eq!(records.len(), 2);
    assert!(records.iter().all(|record| record.built()));
    assert!(records.iter().all(|record| record.cost_bps > 0.0));
    assert!(records.iter().all(|record| record.slippage_bps >= 0.0));
}

#[test]
fn counterfactual_builder_handles_missing_alignment_estimation_and_no_lookahead() {
    let (_, linked_safe) = official_committee_support::build_controlled_linked_pack(
        "counterfactual-builder-edge",
        true,
    );
    let missing = CommitteeCounterfactualBuilder::default().build_records(
        &linked_safe.linked_rows[0],
        None,
        &CommitteeCounterfactualBuildConfig::default(),
    );
    assert!(missing.iter().all(|record| {
        record.build_status == CounterfactualBuildStatus::UnavailableNoCandleData
    }));

    let mismatch_path = official_committee_support::write_candle_series(
        "counterfactual-builder-edge-mismatch",
        "AAPL",
        1_800_000_000_000,
        1.0,
    );
    let mismatch_series =
        load_local_candle_series_map(&[mismatch_path.display().to_string()]).expect("series");
    let mismatch = CommitteeCounterfactualBuilder::default().build_records(
        &linked_safe.linked_rows[0],
        mismatch_series.get("AAPL"),
        &CommitteeCounterfactualBuildConfig::default(),
    );
    assert!(mismatch.iter().all(|record| {
        record.build_status == CounterfactualBuildStatus::UnavailableNoTimestampMatch
    }));

    let estimated = CommitteeCounterfactualBuilder::default().build_records(
        &linked_safe.linked_rows[0],
        mismatch_series.get("AAPL"),
        &CommitteeCounterfactualBuildConfig {
            allow_estimated_when_missing_candles: true,
            ..CommitteeCounterfactualBuildConfig::default()
        },
    );
    assert!(estimated.iter().all(|record| record.diagnostic_only));
    assert!(estimated.iter().all(|record| {
        record.build_status == CounterfactualBuildStatus::EstimatedDiagnosticOnly
    }));

    let short_path = common::output_dir("counterfactual-builder-short").join("candles.json");
    official_committee_support::write_json(
        &short_path,
        serde_json::json!({
            "symbol": "AAPL",
            "timeframe": "OneDay",
            "candles": [
                {"timestamp_ms": 1700000000000u64, "open": 100.0, "high": 101.0, "low": 99.0, "close": 100.5, "volume": 1000.0, "spread_bps": 4.0},
                {"timestamp_ms": 1700000000001u64, "open": 100.5, "high": 101.0, "low": 99.5, "close": 100.0, "volume": 1001.0, "spread_bps": 4.0}
            ]
        }),
    );
    let short_series =
        load_local_candle_series_map(&[short_path.display().to_string()]).expect("series");
    let short = CommitteeCounterfactualBuilder::default().build_records(
        &linked_safe.linked_rows[0],
        short_series.get("AAPL"),
        &CommitteeCounterfactualBuildConfig::default(),
    );
    assert!(short.iter().all(|record| {
        record.build_status == CounterfactualBuildStatus::UnavailableWrongHorizon
    }));

    let (_, linked_unsafe) = official_committee_support::build_controlled_linked_pack(
        "counterfactual-builder-unsafe",
        false,
    );
    let unsafe_result = CommitteeCounterfactualBuilder::default().build_records(
        &linked_unsafe.linked_rows[0],
        short_series.get("AAPL"),
        &CommitteeCounterfactualBuildConfig::default(),
    );
    assert!(
        unsafe_result.iter().all(|record| {
            record.build_status == CounterfactualBuildStatus::RejectedNoLookahead
        })
    );
}

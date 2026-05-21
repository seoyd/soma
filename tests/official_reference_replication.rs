mod common;
#[path = "support/official_committee_support.rs"]
mod official_committee_support;

use soma_zero::{
    OfficialCandleCoverageRunner, OfficialEvidenceReplicationConfig,
    OfficialReferenceReplicationRunner, OfficialReferenceReplicationStatus,
    OfficialRowInjectionResult, ProviderMarket,
};

fn row_injection_result(
    rows: Vec<soma_zero::CommitteeScenarioRow>,
    official_row_count: usize,
    non_crypto_official_row_count: usize,
    crypto_only_row_count: usize,
) -> OfficialRowInjectionResult {
    OfficialRowInjectionResult {
        injected_rows: rows,
        skipped_rows: Vec::new(),
        official_row_count,
        non_crypto_official_row_count,
        crypto_only_row_count,
        skipped_missing_provenance: 0,
        skipped_missing_preflight: 0,
        skipped_research_only: 0,
        skipped_fixture: 0,
        skipped_summary_derived: 0,
        reason_codes: Vec::new(),
    }
}

fn official_row(name: &str) -> soma_zero::CommitteeScenarioRow {
    let mut row = official_committee_support::scenario_row(name, 0, "AAPL", 1_700_000_000_000);
    row.provenance_summary = "row-level-provenance: official-api-collected".to_string();
    row
}

#[test]
fn reference_replication_blocks_when_candles_are_missing() {
    let config = OfficialEvidenceReplicationConfig {
        replication_id: "reference-blocked".to_string(),
        output_root: common::output_dir("reference-blocked")
            .display()
            .to_string(),
        ..OfficialEvidenceReplicationConfig::default()
    };
    let injection = row_injection_result(vec![official_row("reference-blocked")], 1, 1, 0);
    let coverage = OfficialCandleCoverageRunner::default()
        .run(&injection.injected_rows, &[])
        .expect("coverage");
    let artifacts = OfficialReferenceReplicationRunner::default()
        .run(&config, &injection, &[], &coverage)
        .expect("blocked report");
    assert_eq!(
        artifacts.report.replication_status,
        OfficialReferenceReplicationStatus::MissingOfficialCandleData
    );
    assert!(artifacts.bundle.is_none());
}

#[test]
fn reference_replication_generates_official_references_and_linked_pack() {
    let config = OfficialEvidenceReplicationConfig {
        replication_id: "reference-official".to_string(),
        output_root: common::output_dir("reference-official")
            .display()
            .to_string(),
        ..OfficialEvidenceReplicationConfig::default()
    };
    let injection = row_injection_result(vec![official_row("reference-official")], 1, 1, 0);
    let candle_path = official_committee_support::write_candle_series(
        "reference-official",
        "AAPL",
        1_700_000_000_000,
        1.0,
    );
    let coverage = OfficialCandleCoverageRunner::default()
        .run(
            &injection.injected_rows,
            &[candle_path.display().to_string()],
        )
        .expect("coverage");
    let artifacts = OfficialReferenceReplicationRunner::default()
        .run(
            &config,
            &injection,
            &[candle_path.display().to_string()],
            &coverage,
        )
        .expect("official references");
    assert_eq!(
        artifacts.report.replication_status,
        OfficialReferenceReplicationStatus::OfficialReferencesGenerated
    );
    assert!(artifacts.report.official_ready_reference_count > 0);
    assert!(artifacts.report.outcome_reference_count > 0);
    assert!(artifacts.report.baseline_reference_count > 0);
    assert!(artifacts.bundle.is_some());
    assert!(artifacts.linked_pack.is_some());
}

#[test]
fn reference_replication_distinguishes_controlled_and_crypto_only_rows() {
    let mut controlled = official_row("reference-controlled");
    controlled.provenance_summary = "controlled-local".to_string();
    let controlled_injection = row_injection_result(vec![controlled], 0, 0, 0);
    let candle_path = official_committee_support::write_candle_series(
        "reference-controlled",
        "AAPL",
        1_700_000_000_000,
        1.0,
    );
    let controlled_coverage = OfficialCandleCoverageRunner::default()
        .run(
            &controlled_injection.injected_rows,
            &[candle_path.display().to_string()],
        )
        .expect("coverage");
    let controlled_artifacts = OfficialReferenceReplicationRunner::default()
        .run(
            &OfficialEvidenceReplicationConfig {
                replication_id: "reference-controlled".to_string(),
                output_root: common::output_dir("reference-controlled-out")
                    .display()
                    .to_string(),
                ..OfficialEvidenceReplicationConfig::default()
            },
            &controlled_injection,
            &[candle_path.display().to_string()],
            &controlled_coverage,
        )
        .expect("controlled refs");
    assert_eq!(
        controlled_artifacts.report.replication_status,
        OfficialReferenceReplicationStatus::ControlledReferencesOnly
    );
    assert!(controlled_artifacts.report.controlled_reference_count > 0);

    let mut crypto = official_row("reference-crypto");
    crypto.market = ProviderMarket::Crypto;
    let crypto_injection = row_injection_result(vec![crypto], 1, 0, 1);
    let crypto_candle_path = official_committee_support::write_candle_series(
        "reference-crypto",
        "AAPL",
        1_700_000_000_000,
        1.0,
    );
    let crypto_coverage = OfficialCandleCoverageRunner::default()
        .run(
            &crypto_injection.injected_rows,
            &[crypto_candle_path.display().to_string()],
        )
        .expect("crypto coverage");
    let crypto_artifacts = OfficialReferenceReplicationRunner::default()
        .run(
            &OfficialEvidenceReplicationConfig {
                replication_id: "reference-crypto".to_string(),
                output_root: common::output_dir("reference-crypto-out")
                    .display()
                    .to_string(),
                allow_crypto_only: true,
                ..OfficialEvidenceReplicationConfig::default()
            },
            &crypto_injection,
            &[crypto_candle_path.display().to_string()],
            &crypto_coverage,
        )
        .expect("crypto refs");
    assert_eq!(
        crypto_artifacts.report.replication_status,
        OfficialReferenceReplicationStatus::CryptoOnlyReferences
    );
    assert!(crypto_artifacts.report.crypto_only_reference_count > 0);
}

#[test]
fn reference_replication_can_still_generate_estimated_references_when_future_window_is_short() {
    let config = OfficialEvidenceReplicationConfig {
        replication_id: "reference-short-window".to_string(),
        output_root: common::output_dir("reference-short-window-out")
            .display()
            .to_string(),
        ..OfficialEvidenceReplicationConfig::default()
    };
    let injection = row_injection_result(vec![official_row("reference-short-window")], 1, 1, 0);
    let short_candle_path = common::output_dir("reference-short-window-series").join("aapl.json");
    std::fs::write(
        &short_candle_path,
        serde_json::to_string_pretty(&serde_json::json!({
            "symbol": "AAPL",
            "timeframe": "OneDay",
            "candles": [
                {"timestamp_ms": 1700000000000u64, "open": 100.0, "high": 101.0, "low": 99.0, "close": 100.5, "volume": 1.0, "spread_bps": 4.0},
                {"timestamp_ms": 1700000000001u64, "open": 101.0, "high": 102.0, "low": 100.0, "close": 101.5, "volume": 1.0, "spread_bps": 4.0},
                {"timestamp_ms": 1700000000002u64, "open": 102.0, "high": 103.0, "low": 101.0, "close": 102.5, "volume": 1.0, "spread_bps": 4.0}
            ]
        }))
        .expect("json"),
    )
    .expect("write short series");
    let coverage = OfficialCandleCoverageRunner::default()
        .run(
            &injection.injected_rows,
            &[short_candle_path.display().to_string()],
        )
        .expect("coverage");
    let artifacts = OfficialReferenceReplicationRunner::default()
        .run(
            &config,
            &injection,
            &[short_candle_path.display().to_string()],
            &coverage,
        )
        .expect("short window refs");
    assert_eq!(
        artifacts.report.replication_status,
        OfficialReferenceReplicationStatus::OfficialReferencesGenerated
    );
    assert!(artifacts.report.official_ready_reference_count > 0);
}

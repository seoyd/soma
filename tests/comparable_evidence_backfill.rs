#[path = "support/candle_coverage_support.rs"]
mod candle_coverage_support;
mod common;

use std::fs;

use soma_zero::{
    ComparableEvidenceBackfillConfig, ComparableEvidenceBackfillRunner,
    ComparableEvidenceSourceClass, EvidenceSourceKind, OfficialCandleCoveragePackConfig,
    PreflightFinalStatus,
};

#[test]
fn comparable_backfill_adds_candle_coverage_without_promoting_source_class() {
    let timestamps = (0..8)
        .map(|index| 1_700_000_000_000 + index * 86_400_000)
        .collect::<Vec<_>>();
    let official_csv =
        candle_coverage_support::write_csv("backfill", "aapl_1d", "AAPL", "1d", &timestamps, false);
    let official_provenance = candle_coverage_support::write_provenance(
        "backfill",
        "aapl_1d",
        EvidenceSourceKind::OfficialApiCollected,
        "official",
        &official_csv,
        true,
    );
    let official_preflight = candle_coverage_support::write_preflight(
        "backfill",
        "aapl_1d",
        "AAPL",
        soma_zero::Timeframe::OneDay,
        &official_csv,
        EvidenceSourceKind::OfficialApiCollected,
        PreflightFinalStatus::ReadyForRealEvidence,
    );
    let official_manifest = candle_coverage_support::write_manifest(
        "backfill",
        "aapl_1d",
        "AAPL",
        soma_zero::Timeframe::OneDay,
        EvidenceSourceKind::OfficialApiCollected,
        &official_csv,
        &timestamps,
    );
    let pack_config = OfficialCandleCoveragePackConfig {
        pack_id: "backfill-pack".to_string(),
        canonical_csv_paths: vec![official_csv.display().to_string()],
        provenance_paths: vec![official_provenance.display().to_string()],
        preflight_report_paths: vec![official_preflight.display().to_string()],
        manifest_paths: vec![official_manifest.display().to_string()],
        output_root: common::output_dir("backfill-pack-out")
            .display()
            .to_string(),
        ..OfficialCandleCoveragePackConfig::default()
    };
    let pack_config_path =
        candle_coverage_support::write_pack_config_file("backfill", &pack_config);

    let bundle_path = candle_coverage_support::write_bundle(
        "backfill-bundle",
        vec![
            candle_coverage_support::comparable_row(
                "official-row",
                "AAPL",
                "1d",
                timestamps[0],
                ComparableEvidenceSourceClass::OfficialNonCrypto,
                true,
                false,
                false,
            ),
            candle_coverage_support::comparable_row(
                "yfinance-row",
                "AAPL",
                "1d",
                timestamps[0],
                ComparableEvidenceSourceClass::YFinanceResearch,
                false,
                true,
                false,
            ),
            candle_coverage_support::comparable_row(
                "summary-row",
                "AAPL",
                "1d",
                timestamps[0],
                ComparableEvidenceSourceClass::OfficialNonCrypto,
                false,
                false,
                true,
            ),
        ],
    );

    let config = ComparableEvidenceBackfillConfig {
        backfill_id: "backfill".to_string(),
        comparable_evidence_bundle_paths: vec![bundle_path.display().to_string()],
        official_candle_coverage_pack_paths: vec![pack_config_path.display().to_string()],
        output_root: common::output_dir("backfill-out").display().to_string(),
        allow_diagnostic_backfill: true,
        ..ComparableEvidenceBackfillConfig::default()
    };
    let result = ComparableEvidenceBackfillRunner::default()
        .run_bundle(&config)
        .expect("run backfill");
    let official_row = result
        .bundle
        .rows
        .iter()
        .find(|row| row.row_id == "official-row")
        .expect("official row");
    assert!(official_row.candle_coverage_available);
    assert!(official_row.candle_official_ready_match);
    assert!(!official_row.outcome_reference_available);

    let research_row = result
        .bundle
        .rows
        .iter()
        .find(|row| row.row_id == "yfinance-row")
        .expect("research row");
    assert!(research_row.candle_coverage_available);
    assert!(!research_row.candle_official_ready_match);
    assert_eq!(
        research_row.source_class,
        ComparableEvidenceSourceClass::YFinanceResearch
    );

    let summary_row = result
        .bundle
        .rows
        .iter()
        .find(|row| row.row_id == "summary-row")
        .expect("summary row");
    assert!(summary_row.summary_derived);
    assert!(!summary_row.row_level);
    assert_eq!(result.report.rows_with_new_candle_match, 3);
    assert_eq!(result.report.rows_still_summary_derived, 1);

    let output = result
        .write_to_dir(&config.output_dir())
        .expect("write output");
    assert!(fs::metadata(output).is_ok());
}

#[test]
fn comparable_backfill_is_deterministic() {
    let bundle_path = candle_coverage_support::write_bundle(
        "backfill-deterministic-bundle",
        vec![candle_coverage_support::comparable_row(
            "det-row",
            "AAPL",
            "1d",
            1_700_000_000_000,
            ComparableEvidenceSourceClass::OfficialNonCrypto,
            true,
            false,
            false,
        )],
    );
    let config = ComparableEvidenceBackfillConfig {
        backfill_id: "backfill-deterministic".to_string(),
        comparable_evidence_bundle_paths: vec![bundle_path.display().to_string()],
        official_candle_coverage_pack_paths: Vec::new(),
        output_root: common::output_dir("backfill-deterministic-out")
            .display()
            .to_string(),
        ..ComparableEvidenceBackfillConfig::default()
    };
    let first = ComparableEvidenceBackfillRunner::default()
        .run_bundle(&config)
        .expect("first");
    let second = ComparableEvidenceBackfillRunner::default()
        .run_bundle(&config)
        .expect("second");
    assert_eq!(first.report.to_text(), second.report.to_text());
}

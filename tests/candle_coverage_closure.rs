#[path = "support/candle_coverage_support.rs"]
mod candle_coverage_support;
mod common;

use std::fs;

use soma_zero::{
    CandleCoverageClosureConfig, CandleCoverageClosureFinalStatus, CandleCoverageClosureRunner,
    ComparableEvidenceBackfillConfig, ComparableEvidenceSourceClass, EvidenceSourceKind,
    OfficialCandleCoveragePackConfig, PreflightFinalStatus,
};

fn write_backfill_config(
    name: &str,
    config: &ComparableEvidenceBackfillConfig,
) -> std::path::PathBuf {
    let path = common::output_dir(&format!("{name}-backfill-config")).join("backfill.toml");
    fs::write(&path, config.to_toml_string().expect("toml")).expect("write backfill config");
    path
}

#[test]
fn candle_coverage_closure_handles_missing_pack_and_improved_pack_statuses() {
    let missing = CandleCoverageClosureRunner::default()
        .run(&CandleCoverageClosureConfig {
            closure_id: "closure-missing".to_string(),
            output_root: common::output_dir("closure-missing").display().to_string(),
            ..CandleCoverageClosureConfig::default()
        })
        .expect("missing closure");
    assert_eq!(
        missing.final_status,
        CandleCoverageClosureFinalStatus::StillMissingOfficialCandles
    );

    let timestamps = (0..8)
        .map(|index| 1_700_000_000_000 + index * 86_400_000)
        .collect::<Vec<_>>();
    let csv = candle_coverage_support::write_csv(
        "closure-improved",
        "aapl_1d",
        "AAPL",
        "1d",
        &timestamps,
        false,
    );
    let provenance = candle_coverage_support::write_provenance(
        "closure-improved",
        "aapl_1d",
        EvidenceSourceKind::OfficialApiCollected,
        "official",
        &csv,
        true,
    );
    let preflight = candle_coverage_support::write_preflight(
        "closure-improved",
        "aapl_1d",
        "AAPL",
        soma_zero::Timeframe::OneDay,
        &csv,
        EvidenceSourceKind::OfficialApiCollected,
        PreflightFinalStatus::ReadyForRealEvidence,
    );
    let manifest = candle_coverage_support::write_manifest(
        "closure-improved",
        "aapl_1d",
        "AAPL",
        soma_zero::Timeframe::OneDay,
        EvidenceSourceKind::OfficialApiCollected,
        &csv,
        &timestamps,
    );
    let pack_config = OfficialCandleCoveragePackConfig {
        pack_id: "closure-pack".to_string(),
        canonical_csv_paths: vec![csv.display().to_string()],
        provenance_paths: vec![provenance.display().to_string()],
        preflight_report_paths: vec![preflight.display().to_string()],
        manifest_paths: vec![manifest.display().to_string()],
        output_root: common::output_dir("closure-pack").display().to_string(),
        ..OfficialCandleCoveragePackConfig::default()
    };
    let pack_config_path =
        candle_coverage_support::write_pack_config_file("closure-improved", &pack_config);
    let bundle_path = candle_coverage_support::write_bundle(
        "closure-improved-bundle",
        vec![candle_coverage_support::comparable_row(
            "closure-row",
            "AAPL",
            "1d",
            timestamps[0],
            ComparableEvidenceSourceClass::OfficialNonCrypto,
            true,
            false,
            false,
        )],
    );
    let backfill_config = ComparableEvidenceBackfillConfig {
        backfill_id: "closure-backfill".to_string(),
        comparable_evidence_bundle_paths: vec![bundle_path.display().to_string()],
        official_candle_coverage_pack_paths: vec![pack_config_path.display().to_string()],
        output_root: common::output_dir("closure-backfill").display().to_string(),
        ..ComparableEvidenceBackfillConfig::default()
    };
    let backfill_config_path = write_backfill_config("closure-improved", &backfill_config);
    let improved = CandleCoverageClosureRunner::default()
        .run(&CandleCoverageClosureConfig {
            closure_id: "closure-improved".to_string(),
            candle_pack_config_path: Some(pack_config_path.display().to_string()),
            backfill_config_path: Some(backfill_config_path.display().to_string()),
            output_root: common::output_dir("closure-improved-out")
                .display()
                .to_string(),
            ..CandleCoverageClosureConfig::default()
        })
        .expect("improved closure");
    assert!(matches!(
        improved.final_status,
        CandleCoverageClosureFinalStatus::OfficialCandleCoverageImproved
            | CandleCoverageClosureFinalStatus::CandleCoverageImproved
    ));
}

#[test]
fn candle_coverage_closure_reports_alignment_and_rerun_summaries() {
    let timestamps = vec![1_700_000_000_000, 1_700_086_400_000, 1_700_172_800_000];
    let csv = candle_coverage_support::write_csv(
        "closure-align",
        "aapl_1d",
        "AAPL",
        "1d",
        &timestamps,
        false,
    );
    let provenance = candle_coverage_support::write_provenance(
        "closure-align",
        "aapl_1d",
        EvidenceSourceKind::OfficialApiCollected,
        "official",
        &csv,
        true,
    );
    let preflight = candle_coverage_support::write_preflight(
        "closure-align",
        "aapl_1d",
        "AAPL",
        soma_zero::Timeframe::OneDay,
        &csv,
        EvidenceSourceKind::OfficialApiCollected,
        PreflightFinalStatus::ReadyForRealEvidence,
    );
    let manifest = candle_coverage_support::write_manifest(
        "closure-align",
        "aapl_1d",
        "AAPL",
        soma_zero::Timeframe::OneDay,
        EvidenceSourceKind::OfficialApiCollected,
        &csv,
        &timestamps,
    );
    let pack_config = OfficialCandleCoveragePackConfig {
        pack_id: "closure-align-pack".to_string(),
        canonical_csv_paths: vec![csv.display().to_string()],
        provenance_paths: vec![provenance.display().to_string()],
        preflight_report_paths: vec![preflight.display().to_string()],
        manifest_paths: vec![manifest.display().to_string()],
        output_root: common::output_dir("closure-align-pack")
            .display()
            .to_string(),
        ..OfficialCandleCoveragePackConfig::default()
    };
    let pack_config_path =
        candle_coverage_support::write_pack_config_file("closure-align", &pack_config);

    let bad_timeframe_bundle = candle_coverage_support::write_bundle(
        "closure-timeframe-bundle",
        vec![candle_coverage_support::comparable_row(
            "timeframe-row",
            "AAPL",
            "1h",
            timestamps[0],
            ComparableEvidenceSourceClass::OfficialNonCrypto,
            true,
            false,
            false,
        )],
    );
    let timeframe_backfill = ComparableEvidenceBackfillConfig {
        backfill_id: "closure-timeframe-backfill".to_string(),
        comparable_evidence_bundle_paths: vec![bad_timeframe_bundle.display().to_string()],
        official_candle_coverage_pack_paths: vec![pack_config_path.display().to_string()],
        output_root: common::output_dir("closure-timeframe-backfill")
            .display()
            .to_string(),
        ..ComparableEvidenceBackfillConfig::default()
    };
    let timeframe_backfill_path = write_backfill_config("closure-timeframe", &timeframe_backfill);
    let timeframe_report = CandleCoverageClosureRunner::default()
        .run(&CandleCoverageClosureConfig {
            closure_id: "closure-timeframe".to_string(),
            candle_pack_config_path: Some(pack_config_path.display().to_string()),
            backfill_config_path: Some(timeframe_backfill_path.display().to_string()),
            output_root: common::output_dir("closure-timeframe")
                .display()
                .to_string(),
            ..CandleCoverageClosureConfig::default()
        })
        .expect("timeframe closure");
    assert_eq!(
        timeframe_report.final_status,
        CandleCoverageClosureFinalStatus::StillNeedBetterTimeframeAlignment
    );

    let bad_timestamp_bundle = candle_coverage_support::write_bundle(
        "closure-timestamp-bundle",
        vec![candle_coverage_support::comparable_row(
            "timestamp-row",
            "AAPL",
            "1d",
            timestamps[0] + 10_000_000,
            ComparableEvidenceSourceClass::OfficialNonCrypto,
            true,
            false,
            false,
        )],
    );
    let timestamp_backfill = ComparableEvidenceBackfillConfig {
        backfill_id: "closure-timestamp-backfill".to_string(),
        comparable_evidence_bundle_paths: vec![bad_timestamp_bundle.display().to_string()],
        official_candle_coverage_pack_paths: vec![pack_config_path.display().to_string()],
        output_root: common::output_dir("closure-timestamp-backfill")
            .display()
            .to_string(),
        ..ComparableEvidenceBackfillConfig::default()
    };
    let timestamp_backfill_path = write_backfill_config("closure-timestamp", &timestamp_backfill);
    let timestamp_report = CandleCoverageClosureRunner::default()
        .run(&CandleCoverageClosureConfig {
            closure_id: "closure-timestamp".to_string(),
            candle_pack_config_path: Some(pack_config_path.display().to_string()),
            backfill_config_path: Some(timestamp_backfill_path.display().to_string()),
            output_root: common::output_dir("closure-timestamp")
                .display()
                .to_string(),
            run_reference_generation: true,
            run_counterfactual_depth_close: true,
            run_core_scorecard_rerun: true,
            ..CandleCoverageClosureConfig::default()
        })
        .expect("timestamp closure");
    assert_eq!(
        timestamp_report.final_status,
        CandleCoverageClosureFinalStatus::StillNeedBetterTimestampAlignment
    );
    assert!(timestamp_report.reference_generation_summary.is_some());
    assert!(
        timestamp_report
            .counterfactual_depth_closure_summary
            .is_some()
    );
    assert!(timestamp_report.core_scorecard_rerun_summary.is_some());
}

#[path = "support/candle_coverage_support.rs"]
mod candle_coverage_support;
mod common;

use std::fs;

use soma_zero::{
    CandleCoverageClosureConfig, CandleCoverageClosureRunner, ComparableEvidenceBackfillConfig,
    ComparableEvidenceBackfillRunner, ComparableEvidenceSourceClass, EvidenceSourceKind,
    OfficialCandleCoveragePack, OfficialCandleCoveragePackConfig, PreflightFinalStatus,
};

#[test]
fn candle_coverage_pack_backfill_and_closure_are_deterministic() {
    let timestamps = (0..8)
        .map(|index| 1_700_000_000_000 + index * 86_400_000)
        .collect::<Vec<_>>();
    let csv = candle_coverage_support::write_csv(
        "deterministic",
        "aapl_1d",
        "AAPL",
        "1d",
        &timestamps,
        false,
    );
    let provenance = candle_coverage_support::write_provenance(
        "deterministic",
        "aapl_1d",
        EvidenceSourceKind::OfficialApiCollected,
        "official",
        &csv,
        true,
    );
    let preflight = candle_coverage_support::write_preflight(
        "deterministic",
        "aapl_1d",
        "AAPL",
        soma_zero::Timeframe::OneDay,
        &csv,
        EvidenceSourceKind::OfficialApiCollected,
        PreflightFinalStatus::ReadyForRealEvidence,
    );
    let manifest = candle_coverage_support::write_manifest(
        "deterministic",
        "aapl_1d",
        "AAPL",
        soma_zero::Timeframe::OneDay,
        EvidenceSourceKind::OfficialApiCollected,
        &csv,
        &timestamps,
    );
    let pack_config = OfficialCandleCoveragePackConfig {
        pack_id: "deterministic-pack".to_string(),
        canonical_csv_paths: vec![csv.display().to_string()],
        provenance_paths: vec![provenance.display().to_string()],
        preflight_report_paths: vec![preflight.display().to_string()],
        manifest_paths: vec![manifest.display().to_string()],
        output_root: common::output_dir("deterministic-pack")
            .display()
            .to_string(),
        ..OfficialCandleCoveragePackConfig::default()
    };
    let first_pack = OfficialCandleCoveragePack::build(&pack_config).expect("first pack");
    let second_pack = OfficialCandleCoveragePack::build(&pack_config).expect("second pack");
    assert_eq!(first_pack.to_text(), second_pack.to_text());

    let pack_config_path =
        candle_coverage_support::write_pack_config_file("deterministic", &pack_config);
    let bundle_path = candle_coverage_support::write_bundle(
        "deterministic-bundle",
        vec![candle_coverage_support::comparable_row(
            "det-row",
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
        backfill_id: "deterministic-backfill".to_string(),
        comparable_evidence_bundle_paths: vec![bundle_path.display().to_string()],
        official_candle_coverage_pack_paths: vec![pack_config_path.display().to_string()],
        output_root: common::output_dir("deterministic-backfill")
            .display()
            .to_string(),
        ..ComparableEvidenceBackfillConfig::default()
    };
    let first_backfill = ComparableEvidenceBackfillRunner::default()
        .run_bundle(&backfill_config)
        .expect("first backfill");
    let second_backfill = ComparableEvidenceBackfillRunner::default()
        .run_bundle(&backfill_config)
        .expect("second backfill");
    assert_eq!(
        first_backfill.report.to_text(),
        second_backfill.report.to_text()
    );

    let closure_output_dir = common::output_dir("deterministic-closure");
    let backfill_path = closure_output_dir.join("backfill.toml");
    fs::write(
        &backfill_path,
        backfill_config.to_toml_string().expect("toml"),
    )
    .expect("write backfill");
    let closure_config = CandleCoverageClosureConfig {
        closure_id: "deterministic-closure".to_string(),
        candle_pack_config_path: Some(pack_config_path.display().to_string()),
        backfill_config_path: Some(backfill_path.display().to_string()),
        output_root: closure_output_dir.display().to_string(),
        ..CandleCoverageClosureConfig::default()
    };
    let first_closure = CandleCoverageClosureRunner::default()
        .run(&closure_config)
        .expect("first closure");
    let second_closure = CandleCoverageClosureRunner::default()
        .run(&closure_config)
        .expect("second closure");
    assert_eq!(first_closure.to_text(), second_closure.to_text());
}

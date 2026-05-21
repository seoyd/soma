mod common;
#[path = "support/official_committee_support.rs"]
mod official_committee_support;

use soma_zero::{
    CommitteeCounterfactualBuildConfig, CommitteeCounterfactualBuilder,
    CommitteeOutcomeCoverageConfig, CommitteeOutcomeCoverageStatus,
    OfficialCommitteeScenarioPackBuilder, build_committee_outcome_coverage_report,
    load_local_candle_series_map,
};

#[test]
fn outcome_coverage_report_counts_official_links_and_counterfactuals() {
    let (pack, linked) =
        official_committee_support::build_controlled_linked_pack("coverage-report-counts", true);
    let candle_path = official_committee_support::write_candle_series(
        "coverage-report-counts",
        "AAPL",
        1_700_000_000_000,
        1.0,
    );
    let series =
        load_local_candle_series_map(&[candle_path.display().to_string()]).expect("series");
    let records = linked
        .linked_rows
        .iter()
        .flat_map(|row| {
            CommitteeCounterfactualBuilder::default().build_records(
                row,
                series.get("AAPL"),
                &CommitteeCounterfactualBuildConfig::default(),
            )
        })
        .collect::<Vec<_>>();
    let report = build_committee_outcome_coverage_report(
        &CommitteeOutcomeCoverageConfig {
            coverage_id: "coverage-report-counts".to_string(),
            reason_codes: vec![],
            ..CommitteeOutcomeCoverageConfig::default()
        },
        &[pack],
        &[linked],
        &records,
    );
    assert_eq!(report.official_rows, 3);
    assert_eq!(report.outcome_linked_rows, 3);
    assert_eq!(report.baseline_linked_rows, 3);
    assert_eq!(report.external_linked_rows, 1);
    assert_eq!(report.no_lookahead_violations, 0);
    assert!(report.no_trade_counterfactuals >= 1);
    assert!(report.risk_denied_counterfactuals >= 1);
    assert_eq!(
        report.coverage_status,
        CommitteeOutcomeCoverageStatus::HealthyCoverage
    );
}

#[test]
fn outcome_coverage_report_maps_yfinance_fixture_and_crypto_conservatively() {
    let yfinance_pack = OfficialCommitteeScenarioPackBuilder::default()
        .build(&official_committee_support::yfinance_pack_config(
            "coverage-report-yf",
        ))
        .expect("yfinance pack");
    let yfinance = build_committee_outcome_coverage_report(
        &CommitteeOutcomeCoverageConfig::default(),
        &[yfinance_pack],
        &[],
        &[],
    );
    assert_eq!(
        yfinance.coverage_status,
        CommitteeOutcomeCoverageStatus::ResearchOnlyCoverage
    );

    let fixture_pack = OfficialCommitteeScenarioPackBuilder::default()
        .build(&official_committee_support::fixture_pack_config(
            "coverage-report-fixture",
        ))
        .expect("fixture pack");
    let fixture = build_committee_outcome_coverage_report(
        &CommitteeOutcomeCoverageConfig::default(),
        &[fixture_pack],
        &[],
        &[],
    );
    assert_eq!(
        fixture.coverage_status,
        CommitteeOutcomeCoverageStatus::FixtureOnlyCoverage
    );

    let crypto_pack = OfficialCommitteeScenarioPackBuilder::default()
        .build(&official_committee_support::crypto_pack_config(
            "coverage-report-crypto",
        ))
        .expect("crypto pack");
    let crypto = build_committee_outcome_coverage_report(
        &CommitteeOutcomeCoverageConfig::default(),
        &[crypto_pack],
        &[],
        &[],
    );
    assert_eq!(
        crypto.coverage_status,
        CommitteeOutcomeCoverageStatus::CryptoOnlyCoverage
    );
}

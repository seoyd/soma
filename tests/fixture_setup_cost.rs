#[path = "support/sprint69_support.rs"]
mod support;

use soma_zero::{FixtureSetupCostKind, FixtureSetupCostReportStatus};

#[test]
fn fixture_setup_cost_report_identifies_repeated_setup_kinds() {
    let bundle = support::run_sprint77_bundle("soma_fixture_setup_cost.toml", "fixture-setup-cost");
    let report = bundle.fixture_setup_cost_report;
    assert!(
        report
            .records
            .iter()
            .any(|record| record.setup_cost_kind == FixtureSetupCostKind::RepeatedJsonLoad)
    );
    assert!(
        report
            .records
            .iter()
            .any(|record| record.setup_cost_kind == FixtureSetupCostKind::RepeatedTomlParse)
    );
    assert!(
        report
            .records
            .iter()
            .any(|record| record.setup_cost_kind == FixtureSetupCostKind::RepeatedOutputDirSetup)
    );
    assert!(report.records.iter().any(
        |record| record.setup_cost_kind == FixtureSetupCostKind::RepeatedSyntheticDatasetBuild
    ));
    assert!(report.dedup_candidate_count > 0);
    assert!(report.cache_candidate_count > 0);
    assert_eq!(
        report.report_status,
        FixtureSetupCostReportStatus::FixtureCostReady
    );
}

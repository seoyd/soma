mod support;

use soma_zero::CounterfactualBackfillFixtureSetupReductionStatus;
use support::sprint69_support as sprint;

#[test]
fn counterfactual_backfill_fixture_setup_reuses_shared_harness() {
    let report = sprint::run_sprint97_bundle(
        "soma_sprint97_counterfactual_backfill_recover.toml",
        "counterfactual-fixture-setup",
    )
    .counterfactual_backfill_fixture_setup_reduction_report;
    assert_eq!(
        report.reduction_status,
        CounterfactualBackfillFixtureSetupReductionStatus::FixtureSetupReduced
    );
    assert!(report.shared_fixture_harness_used);
}

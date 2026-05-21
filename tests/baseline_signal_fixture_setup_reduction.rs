mod support;

use soma_zero::BaselineSignalFixtureSetupReductionStatus;
use support::sprint69_support as sprint;

#[test]
fn baseline_signal_fixture_setup_reduction_stays_minimal_and_deterministic() {
    let bundle = sprint::run_sprint96_bundle(
        "soma_sprint96_baseline_signal_recover.toml",
        "baseline-signal-fixture-setup-reduction",
    );
    let report = bundle.baseline_signal_fixture_setup_reduction_report;
    assert_eq!(
        report.reduction_status,
        BaselineSignalFixtureSetupReductionStatus::FixtureSetupReduced
    );
    assert_eq!(report.duplicate_output_dirs_removed, 1);
    assert!(report.shared_fixture_harness_used);
    assert!(report.deterministic_output_preserved);
}

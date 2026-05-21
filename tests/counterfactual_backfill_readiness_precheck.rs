mod support;

use soma_zero::CounterfactualBackfillReadinessPrecheckStatus;
use support::sprint69_support as sprint;

#[test]
fn counterfactual_backfill_readiness_precheck_stays_ready() {
    let bundle = sprint::run_sprint96_bundle(
        "soma_sprint96_baseline_signal_recover.toml",
        "counterfactual-backfill-readiness-precheck",
    );
    let report = bundle.counterfactual_backfill_readiness_precheck_report;
    assert_eq!(
        report.precheck_status,
        CounterfactualBackfillReadinessPrecheckStatus::CounterfactualBackfillPrecheckReady
    );
    assert!(report.counterfactual_backfill_family_present);
    assert!(report.counterfactual_backfill_suite_present);
    assert!(report.no_lookahead_checks_present);
}

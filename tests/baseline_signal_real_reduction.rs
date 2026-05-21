mod support;

use soma_zero::{BaselineSignalRealReductionAction, BaselineSignalRealReductionPlanStatus};
use support::sprint69_support as sprint;

#[test]
fn baseline_signal_real_reduction_plan_is_ready_and_conservative() {
    let bundle = sprint::run_sprint96_bundle(
        "soma_sprint96_baseline_signal_recover.toml",
        "baseline-signal-real-reduction",
    );
    let plan = bundle.baseline_signal_real_reduction_plan;
    assert_eq!(
        plan.plan_status,
        BaselineSignalRealReductionPlanStatus::BaselineReductionPlanReady
    );
    assert_eq!(plan.target_files, vec!["tests/baseline_signal_suite.rs"]);
    assert!(
        plan.actions
            .contains(&BaselineSignalRealReductionAction::VerifyGroupedSuiteCoverage)
    );
    assert!(
        plan.actions
            .contains(&BaselineSignalRealReductionAction::KeepSeparateForIsolation)
    );
}

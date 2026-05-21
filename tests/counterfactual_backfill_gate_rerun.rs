mod support;

use soma_zero::{
    CounterfactualBackfillFullGateRerunStatus, CounterfactualBackfillNoRunGateRerunStatus,
};
use support::sprint69_support as sprint;

#[test]
fn counterfactual_backfill_gate_reruns_stay_not_run_in_smoke_fixture() {
    let bundle = sprint::run_sprint97_bundle(
        "soma_sprint97_counterfactual_backfill_recover.toml",
        "counterfactual-gate-rerun",
    );
    assert_eq!(
        bundle
            .counterfactual_backfill_no_run_gate_rerun_report
            .status,
        CounterfactualBackfillNoRunGateRerunStatus::NotRun
    );
    assert_eq!(
        bundle.counterfactual_backfill_full_gate_rerun_report.status,
        CounterfactualBackfillFullGateRerunStatus::NotRun
    );
}

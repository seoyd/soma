mod support;

use soma_zero::RealNoRunGateAttemptV12Status;
use support::sprint69_support as sprint;

#[test]
fn real_no_run_gate_attempt_v12_stays_not_run_in_smoke_fixture() {
    let report = sprint::run_sprint97_bundle(
        "soma_sprint97_counterfactual_backfill_recover.toml",
        "real-no-run-v12",
    )
    .real_no_run_gate_attempt_v12;
    assert_eq!(report.no_run_status, RealNoRunGateAttemptV12Status::NotRun);
}

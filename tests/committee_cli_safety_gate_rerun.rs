mod support;

use soma_zero::{
    CommitteeCliSafetyFullGateRerunStatus, CommitteeCliSafetyNoRunGateRerunStatus,
    Sprint95CommitteeCliSafetyRecoveryRunner,
};
use support::sprint69_support as sprint;

#[test]
fn rerun_reports_stay_not_run_by_default() {
    let config = sprint::sprint95_config_from_example(
        "soma_committee_cli_safety_no_run_rerun.toml",
        "committee-cli-safety-no-run",
    );
    let no_run = Sprint95CommitteeCliSafetyRecoveryRunner::default()
        .run_committee_cli_safety_no_run_rerun(&config)
        .expect("no-run");
    assert_eq!(
        no_run.status,
        CommitteeCliSafetyNoRunGateRerunStatus::NotRun
    );

    let full = Sprint95CommitteeCliSafetyRecoveryRunner::default()
        .run_committee_cli_safety_full_gate_rerun(&sprint::sprint95_config_from_example(
            "soma_committee_cli_safety_full_gate_rerun.toml",
            "committee-cli-safety-full-gate",
        ))
        .expect("full");
    assert_eq!(full.status, CommitteeCliSafetyFullGateRerunStatus::NotRun);
}

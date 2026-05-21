mod support;

use soma_zero::{CommitteeCliSafetyDeterminismStatus, Sprint95CommitteeCliSafetyRecoveryRunner};
use support::sprint69_support as sprint;

#[test]
fn determinism_report_stays_preserved() {
    let report = Sprint95CommitteeCliSafetyRecoveryRunner::default()
        .run_committee_cli_safety_determinism_preservation(&sprint::sprint95_config_from_example(
            "soma_committee_cli_safety_determinism_preservation.toml",
            "committee-cli-safety-determinism",
        ))
        .expect("report");
    assert_eq!(
        report.determinism_status,
        CommitteeCliSafetyDeterminismStatus::DeterminismPreserved
    );
}

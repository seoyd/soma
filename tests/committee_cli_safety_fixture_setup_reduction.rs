mod support;

use soma_zero::{
    CommitteeCliSafetyFixtureSetupReductionStatus, Sprint95CommitteeCliSafetyRecoveryRunner,
};
use support::sprint69_support as sprint;

#[test]
fn fixture_setup_reduction_uses_shared_harness() {
    let report = Sprint95CommitteeCliSafetyRecoveryRunner::default()
        .run_committee_cli_safety_fixture_setup_reduction(&sprint::sprint95_config_from_example(
            "soma_committee_cli_safety_fixture_setup_reduction.toml",
            "committee-cli-safety-fixture-setup",
        ))
        .expect("report");
    assert_eq!(
        report.reduction_status,
        CommitteeCliSafetyFixtureSetupReductionStatus::FixtureSetupReduced
    );
    assert!(report.shared_fixture_harness_used);
    assert!(report.deterministic_output_preserved);
}

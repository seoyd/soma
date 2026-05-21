mod support;

use soma_zero::{
    CommitteeCliSafetyForbiddenCommandPreservationReport, CommitteeCliSafetyForbiddenCommandStatus,
    Sprint95CommitteeCliSafetyRecoveryRunner,
};
use support::{shared_fixture_harness as harness, sprint69_support as sprint};

#[test]
fn forbidden_command_preservation_matches_expected_fixture() {
    let report = Sprint95CommitteeCliSafetyRecoveryRunner::default()
        .run_committee_cli_safety_forbidden_command_preservation(
            &sprint::sprint95_config_from_example(
                "soma_committee_cli_safety_forbidden_command_preservation.toml",
                "committee-cli-safety-forbidden-command",
            ),
        )
        .expect("report");
    let mut expected = harness::load_json_fixture::<
        CommitteeCliSafetyForbiddenCommandPreservationReport,
    >(sprint::example_path(
        "sprint95_data/committee_cli_safety_forbidden_command_expected.json",
    ));
    expected.report_id = report.report_id.clone();
    assert_eq!(report, expected);
    assert_eq!(
        report.forbidden_command_status,
        CommitteeCliSafetyForbiddenCommandStatus::ForbiddenCommandsAbsent
    );
}

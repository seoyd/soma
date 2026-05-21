mod support;

use soma_zero::{
    CommitteeCliSafetyRemotePathPreservationReport, CommitteeCliSafetyRemotePathStatus,
    Sprint95CommitteeCliSafetyRecoveryRunner,
};
use support::{shared_fixture_harness as harness, sprint69_support as sprint};

#[test]
fn remote_path_preservation_matches_expected_fixture() {
    let report = Sprint95CommitteeCliSafetyRecoveryRunner::default()
        .run_committee_cli_safety_remote_path_preservation(&sprint::sprint95_config_from_example(
            "soma_committee_cli_safety_remote_path_preservation.toml",
            "committee-cli-safety-remote-path",
        ))
        .expect("report");
    let mut expected = harness::load_json_fixture::<CommitteeCliSafetyRemotePathPreservationReport>(
        sprint::example_path("sprint95_data/committee_cli_safety_remote_path_expected.json"),
    );
    expected.report_id = report.report_id.clone();
    assert_eq!(report, expected);
    assert_eq!(
        report.remote_path_status,
        CommitteeCliSafetyRemotePathStatus::RemotePathRejectionPreserved
    );
}

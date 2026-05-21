mod support;

use soma_zero::{
    CommitteeCliSafetyHelpTextPreservationReport, CommitteeCliSafetyHelpTextStatus,
    Sprint95CommitteeCliSafetyRecoveryRunner,
};
use support::{shared_fixture_harness as harness, sprint69_support as sprint};

#[test]
fn help_text_preservation_matches_expected_fixture() {
    let report = Sprint95CommitteeCliSafetyRecoveryRunner::default()
        .run_committee_cli_safety_help_text_preservation(&sprint::sprint95_config_from_example(
            "soma_committee_cli_safety_help_text_preservation.toml",
            "committee-cli-safety-help-text",
        ))
        .expect("report");
    let mut expected = harness::load_json_fixture::<CommitteeCliSafetyHelpTextPreservationReport>(
        sprint::example_path("sprint95_data/committee_cli_safety_help_text_expected.json"),
    );
    expected.report_id = report.report_id.clone();
    assert_eq!(report, expected);
    assert_eq!(
        report.help_text_status,
        CommitteeCliSafetyHelpTextStatus::HelpTextPreserved
    );
}

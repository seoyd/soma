mod support;

use soma_zero::{
    CommitteeCliSafetyPersonaExpansionGuardReport, CommitteeCliSafetyPersonaGuardStatus,
    Sprint95CommitteeCliSafetyRecoveryRunner,
};
use support::{shared_fixture_harness as harness, sprint69_support as sprint};

#[test]
fn persona_expansion_guard_matches_expected_fixture() {
    let report = Sprint95CommitteeCliSafetyRecoveryRunner::default()
        .run_committee_cli_safety_persona_expansion_guard(&sprint::sprint95_config_from_example(
            "soma_committee_cli_safety_persona_expansion_guard.toml",
            "committee-cli-safety-persona-guard",
        ))
        .expect("report");
    let mut expected = harness::load_json_fixture::<CommitteeCliSafetyPersonaExpansionGuardReport>(
        sprint::example_path("sprint95_data/committee_cli_safety_persona_guard_expected.json"),
    );
    expected.report_id = report.report_id.clone();
    assert_eq!(report, expected);
    assert_eq!(
        report.persona_guard_status,
        CommitteeCliSafetyPersonaGuardStatus::PersonaExpansionGuardPreserved
    );
}

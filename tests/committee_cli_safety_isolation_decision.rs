mod support;

use soma_zero::{
    CommitteeCliSafetyIsolationDecision, CommitteeCliSafetyIsolationDecisionStatus,
    Sprint95CommitteeCliSafetyRecoveryRunner,
};
use support::{shared_fixture_harness as harness, sprint69_support as sprint};

#[test]
fn committee_cli_safety_isolation_decision_matches_expected_fixture() {
    let report = Sprint95CommitteeCliSafetyRecoveryRunner::default()
        .run_committee_cli_safety_isolation_decision(&sprint::sprint95_config_from_example(
            "soma_committee_cli_safety_isolation_decision.toml",
            "committee-cli-safety-isolation-decision",
        ))
        .expect("report");
    let mut expected = harness::load_json_fixture::<CommitteeCliSafetyIsolationDecision>(
        sprint::example_path("sprint95_data/committee_cli_safety_isolation_decision_expected.json"),
    );
    expected.decision_id = report.decision_id.clone();
    assert_eq!(report, expected);
    assert_eq!(
        report.decision_status,
        CommitteeCliSafetyIsolationDecisionStatus::KeepPermanentIsolatedSentinel
    );
}

#[test]
fn isolation_status_variants_remain_public() {
    let statuses = [
        CommitteeCliSafetyIsolationDecisionStatus::KeepPermanentIsolatedSentinel,
        CommitteeCliSafetyIsolationDecisionStatus::SafeToMergeIntoWorkspaceCliSafetySuite,
        CommitteeCliSafetyIsolationDecisionStatus::SafeToRepresentInWorkspaceSafetyGuardSuite,
        CommitteeCliSafetyIsolationDecisionStatus::SplitIntoSmallerIsolatedSentinels,
        CommitteeCliSafetyIsolationDecisionStatus::UnsafeToMerge,
    ];
    assert_eq!(statuses.len(), 5);
}

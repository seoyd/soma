mod support;

use soma_zero::{
    CommitteeCliSafetyRuntimeDeferredPreservationReport, CommitteeCliSafetyRuntimeDeferredStatus,
    Sprint95CommitteeCliSafetyRecoveryRunner,
};
use support::{shared_fixture_harness as harness, sprint69_support as sprint};

#[test]
fn runtime_deferred_preservation_matches_expected_fixture() {
    let report = Sprint95CommitteeCliSafetyRecoveryRunner::default()
        .run_committee_cli_safety_runtime_deferred_preservation(
            &sprint::sprint95_config_from_example(
                "soma_committee_cli_safety_runtime_deferred_preservation.toml",
                "committee-cli-safety-runtime-deferred",
            ),
        )
        .expect("report");
    let mut expected = harness::load_json_fixture::<
        CommitteeCliSafetyRuntimeDeferredPreservationReport,
    >(sprint::example_path(
        "sprint95_data/committee_cli_safety_runtime_deferred_expected.json",
    ));
    expected.report_id = report.report_id.clone();
    assert_eq!(report, expected);
    assert_eq!(
        report.runtime_deferred_status,
        CommitteeCliSafetyRuntimeDeferredStatus::RuntimeDeferredPreserved
    );
}

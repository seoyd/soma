mod support;

use soma_zero::DeferredObservationSelectionReportV1;
use support::sprint117_support::{read_fixture, run_sprint117};

#[test]
fn deferred_observation_selection_and_plan_are_deterministic() {
    let bundle = run_sprint117(
        "soma_deferred_observation_selection_v1.toml",
        "deferred-observation-selection-v1",
    );
    let expected: DeferredObservationSelectionReportV1 =
        read_fixture("sprint117_data/deferred_observation_selection_expected.json");
    assert_eq!(bundle.deferred_observation_selection_report_v1, expected);
    assert_eq!(
        bundle
            .deferred_observation_execution_plan_v1
            .execution_order,
        vec!["RealCargoJson", "RealNoRun", "RealFullWorkspace"]
    );
}

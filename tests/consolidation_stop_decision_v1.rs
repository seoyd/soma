mod support;

use soma_zero::ConsolidationStopDecisionReportV1;
use support::sprint115_support::{read_fixture, run_sprint115};

#[test]
fn consolidation_stop_decision_v1_matches_expected() {
    let bundle = run_sprint115(
        "soma_consolidation_stop_decision_v1.toml",
        "consolidation-stop-decision-v1",
    );
    let expected: ConsolidationStopDecisionReportV1 =
        read_fixture("sprint115_data/consolidation_stop_decision_expected.json");
    assert_eq!(bundle.consolidation_stop_decision_report_v1, expected);
    assert!(
        bundle
            .consolidation_stop_decision_report_v1
            .stop_recommended
    );
}

mod support;

use serde_json::to_value;
use support::sprint103_support::{read_fixture, run_sprint103};

#[test]
fn paper_notrade_justification_matches_expected_fixture() {
    let bundle = run_sprint103(
        "soma_paper_notrade_justification.toml",
        "paper-notrade-justification",
    );
    let actual = to_value(&bundle.paper_notrade_justification_report).expect("actual");
    let expected: serde_json::Value =
        read_fixture("sprint103_data/no_trade_justification_expected.json");
    assert_eq!(actual, expected);
    assert!(
        bundle
            .paper_notrade_justification_report
            .no_trade_not_failure
    );
    assert!(
        !bundle
            .paper_need_more_evidence_justification_report
            .blocking_for_paper_rotation
    );
}

mod support;

use serde_json::to_value;
use support::sprint103_support::{read_fixture, run_sprint103};

#[test]
fn proposal_run_warning_closure_matches_expected_fixture() {
    let bundle = run_sprint103(
        "soma_sprint103_paper_rotation_close.toml",
        "proposal_run_warning_closure",
    );
    let actual = to_value(&bundle.proposal_run_warning_closure_report).expect("actual");
    let expected: serde_json::Value =
        read_fixture("sprint103_data/proposal_run_closure_expected.json");
    assert_eq!(actual, expected);
}

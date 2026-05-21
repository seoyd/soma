mod support;

use serde_json::to_value;
use support::sprint103_support::{read_fixture, run_sprint103};

#[test]
fn risk_governor_handoff_warning_closure_v2_matches_expected_fixture() {
    let bundle = run_sprint103(
        "soma_sprint103_paper_rotation_close.toml",
        "risk_governor_handoff_warning_closure_v2",
    );
    let actual = to_value(&bundle.risk_governor_handoff_warning_closure_report_v2).expect("actual");
    let expected: serde_json::Value =
        read_fixture("sprint103_data/risk_handoff_closure_expected.json");
    assert_eq!(actual, expected);
}

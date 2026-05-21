mod support;

use serde_json::to_value;
use support::sprint103_support::{read_fixture, run_sprint103};

#[test]
fn control_tower_paper_rotation_closure_panel_matches_expected_fixture() {
    let bundle = run_sprint103(
        "soma_sprint103_paper_rotation_close.toml",
        "control_tower_paper_rotation_closure_panel",
    );
    let actual = to_value(&bundle.control_tower_paper_rotation_closure_panel).expect("actual");
    let expected: serde_json::Value =
        read_fixture("sprint103_data/control_tower_paper_rotation_closure_expected.json");
    assert_eq!(actual, expected);
}

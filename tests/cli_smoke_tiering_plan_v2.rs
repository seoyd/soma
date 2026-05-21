mod support;

use support::sprint106_support::{read_fixture, run_sprint106};

#[test]
fn cli_smoke_tiering_preserves_safety_commands() {
    let bundle = run_sprint106(
        "soma_cli_smoke_tiering_plan_v2.toml",
        "cli_smoke_tiering_plan_v2",
    );
    let actual = serde_json::to_value(&bundle.cli_smoke_tiering_plan_v2).expect("actual");
    let expected: serde_json::Value =
        read_fixture("sprint106_data/cli_smoke_tiering_expected.json");
    assert_eq!(actual, expected);
    assert!(bundle.cli_smoke_tiering_plan_v2.safety_commands_preserved);
}

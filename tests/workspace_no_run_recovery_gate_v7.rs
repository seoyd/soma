mod support;

use support::sprint106_support::{read_fixture, run_sprint106};

#[test]
fn workspace_no_run_recovery_gate_stays_conservative() {
    let bundle = run_sprint106(
        "soma_workspace_no_run_recovery_gate_v7.toml",
        "workspace_no_run_recovery_gate_v7",
    );
    let actual = serde_json::to_value(&bundle.workspace_no_run_recovery_gate_v7).expect("actual");
    let expected: serde_json::Value =
        read_fixture("sprint106_data/no_run_recovery_gate_expected.json");
    assert_eq!(actual, expected);
    assert!(!bundle.workspace_no_run_recovery_gate_v7.no_run_recovered);
}

mod support;

use support::sprint106_support::{read_fixture, run_sprint106};

#[test]
fn binary_inventory_lists_sentinels_and_cli_targets() {
    let bundle = run_sprint106(
        "soma_test_binary_inventory_v3.toml",
        "test_binary_inventory_v3",
    );
    let actual = serde_json::to_value(&bundle.test_binary_inventory_report_v3).expect("actual");
    let expected: serde_json::Value =
        read_fixture("sprint106_data/test_binary_inventory_expected.json");
    assert_eq!(actual, expected);
    assert!(
        !bundle
            .test_binary_inventory_report_v3
            .safety_sentinels
            .is_empty()
    );
    assert!(
        !bundle
            .test_binary_inventory_report_v3
            .cli_safety_targets
            .is_empty()
    );
}

mod support;

use support::sprint106_support::{read_fixture, run_sprint106};

#[test]
fn safe_consolidation_plan_preserves_sentinels_and_blocks_assertion_deletion() {
    let bundle = run_sprint106(
        "soma_safe_test_binary_consolidation_plan_v2.toml",
        "safe_test_binary_consolidation_plan_v2",
    );
    let actual =
        serde_json::to_value(&bundle.safe_test_binary_consolidation_plan_v2).expect("actual");
    let expected: serde_json::Value =
        read_fixture("sprint106_data/safe_consolidation_plan_expected.json");
    assert_eq!(actual, expected);
    assert!(
        bundle
            .safe_test_binary_consolidation_plan_v2
            .assertions_to_preserve
            .iter()
            .any(|item| item.contains("committee CLI safety"))
    );
}

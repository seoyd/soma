mod support;

use support::sprint106_support::{read_fixture, run_sprint106};

#[test]
fn explosion_attribution_detects_repeated_families() {
    let bundle = run_sprint106(
        "soma_test_binary_explosion_attribution.toml",
        "test_binary_explosion_attribution",
    );
    let actual =
        serde_json::to_value(&bundle.test_binary_explosion_attribution_report).expect("actual");
    let expected: serde_json::Value =
        read_fixture("sprint106_data/test_binary_explosion_expected.json");
    assert_eq!(actual, expected);
    assert!(
        !bundle
            .test_binary_explosion_attribution_report
            .high_risk_sentinel_families
            .is_empty()
    );
}

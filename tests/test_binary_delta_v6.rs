mod support;

use support::sprint109_support::{read_fixture, run_sprint109};

#[test]
fn test_binary_delta_v5_matches_expected_fixture() {
    let bundle = run_sprint109("soma_test_binary_delta_v6.toml", "test-binary-delta-v6");
    let actual = serde_json::to_value(&bundle.test_binary_delta_report_v6).expect("actual");
    let expected: serde_json::Value =
        read_fixture("sprint109_data/test_binary_delta_v6_expected.json");
    assert_eq!(actual, expected);
    assert_eq!(bundle.test_binary_delta_report_v6.binary_delta, Some(-1));
    assert_eq!(
        bundle
            .test_binary_delta_report_v6
            .cumulative_sample_backed_delta,
        Some(-3)
    );
    assert!(
        !bundle
            .measured_or_sample_backed_delta_gate_v3
            .can_claim_measured_reduction
    );
}

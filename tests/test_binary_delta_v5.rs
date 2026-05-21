mod support;

use support::sprint108_support::{read_fixture, run_sprint108};

#[test]
fn test_binary_delta_v5_matches_expected_fixture() {
    let bundle = run_sprint108("soma_test_binary_delta_v5.toml", "test-binary-delta-v5");
    let actual = serde_json::to_value(&bundle.test_binary_delta_report_v5).expect("actual");
    let expected: serde_json::Value =
        read_fixture("sprint108_data/test_binary_delta_v5_expected.json");
    assert_eq!(actual, expected);
    assert_eq!(bundle.test_binary_delta_report_v5.binary_delta, Some(-1));
    assert_eq!(
        bundle
            .test_binary_delta_report_v5
            .cumulative_sample_backed_delta,
        -2
    );
    assert!(
        !bundle
            .measured_or_sample_backed_delta_gate_v2
            .can_claim_measured_reduction
    );
}

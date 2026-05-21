mod support;

use support::sprint107_support::{read_fixture, run_sprint107};

#[test]
fn test_binary_delta_is_sample_backed_not_measured() {
    let bundle = run_sprint107("soma_test_binary_delta_v4.toml", "test-binary-delta-v4");
    let actual = serde_json::to_value(&bundle.test_binary_delta_report_v4).expect("actual");
    let expected: serde_json::Value =
        read_fixture("sprint107_data/test_binary_delta_v4_expected.json");
    assert_eq!(actual, expected);
    assert!(bundle.test_binary_delta_report_v4.sample_backed);
    assert!(
        !bundle
            .measured_or_sample_backed_delta_gate_v1
            .can_claim_measured_reduction
    );
}

mod support;

use support::sprint110_support::run_sprint110;

#[test]
fn test_binary_delta_v7_threads_prior_patch_deltas() {
    let bundle = run_sprint110("soma_test_binary_delta_v7.toml", "test-binary-delta-v7");
    let report = bundle.test_binary_delta_report_v7;
    assert_eq!(report.sprint107_delta, Some(-1));
    assert_eq!(report.sprint108_delta, Some(-1));
    assert_eq!(report.sprint109_delta, Some(-1));
    assert_eq!(report.sprint110_delta, Some(-1));
    assert_eq!(report.binary_delta, Some(-1));
    assert_eq!(report.cumulative_sample_backed_delta, Some(-4));
    assert!(report.sample_backed);
    assert!(!report.measured);
    assert!(
        !bundle
            .measured_or_sample_backed_delta_gate_v4
            .can_claim_measured_reduction
    );
}

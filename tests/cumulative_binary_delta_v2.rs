mod support;

use support::sprint110_support::run_sprint110;

#[test]
fn cumulative_binary_delta_v2_sums_sample_backed_reductions() {
    let bundle = run_sprint110(
        "soma_cumulative_binary_delta_v2.toml",
        "cumulative-binary-delta-v2",
    );
    let report = bundle.cumulative_binary_delta_report_v2;
    assert_eq!(report.patch_count, 4);
    assert_eq!(report.sample_backed_deltas, vec![-1, -1, -1, -1]);
    assert_eq!(report.cumulative_sample_backed_delta, -4);
    assert!(!report.measured_claim_allowed);
}

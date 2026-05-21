mod support;

use soma_zero::SccacheLocalOnlyPolicyReportV1;
use support::sprint112_support::{read_fixture, run_sprint112};

#[test]
fn sccache_local_only_policy_blocks_remote_secret_and_overclaim() {
    let bundle = run_sprint112("soma_sccache_local_only_policy_v1.toml", "sccache-policy");
    let expected: SccacheLocalOnlyPolicyReportV1 =
        read_fixture("sprint112_data/sccache_policy_expected.json");
    assert_eq!(bundle.sccache_local_only_policy_report_v1, expected);
    assert!(
        bundle
            .sccache_local_only_policy_report_v1
            .local_only_required
    );
    assert!(
        bundle
            .sccache_local_only_policy_report_v1
            .remote_cache_forbidden
    );
    assert!(
        bundle
            .sccache_local_only_policy_report_v1
            .secret_cache_forbidden
    );
    assert!(
        bundle
            .sccache_local_only_policy_report_v1
            .cache_failure_must_not_hide_failure
    );
    assert!(!bundle.sccache_effect_estimate_report_v1.can_claim_speedup);
}

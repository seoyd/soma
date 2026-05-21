mod support;

use soma_zero::RemainingSafeConsolidationCandidatePoolReport;
use support::sprint111_support::{read_fixture, run_sprint111};

#[test]
fn remaining_safe_candidate_pool_matches_fixture() {
    let bundle = run_sprint111(
        "soma_remaining_safe_candidate_pool.toml",
        "remaining-safe-candidate-pool",
    );
    let expected: RemainingSafeConsolidationCandidatePoolReport =
        read_fixture("sprint111_data/remaining_candidate_pool_expected.json");
    assert_eq!(
        bundle.remaining_safe_consolidation_candidate_pool_report,
        expected
    );
    assert!(
        !bundle
            .remaining_safe_consolidation_candidate_pool_report
            .low_risk_candidates
            .is_empty()
    );
    assert!(
        !bundle
            .remaining_safe_consolidation_candidate_pool_report
            .sentinel_candidates_excluded
            .is_empty()
    );
}

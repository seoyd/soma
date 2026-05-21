mod support;

use soma_zero::ConsolidationPausedCarryForwardReport;
use support::sprint116_support::{read_fixture, run_sprint116};

#[test]
fn consolidation_paused_carry_forward_matches_expected() {
    let bundle = run_sprint116(
        "soma_consolidation_paused_carry_forward.toml",
        "consolidation-paused-carry-forward",
    );
    let expected: ConsolidationPausedCarryForwardReport =
        read_fixture("sprint116_data/consolidation_paused_carry_forward_expected.json");
    assert_eq!(bundle.consolidation_paused_carry_forward_report, expected);
    assert!(
        bundle
            .consolidation_paused_carry_forward_report
            .consolidation_paused
    );
}

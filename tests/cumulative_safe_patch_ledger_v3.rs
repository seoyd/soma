mod support;

use soma_zero::CumulativeSafePatchLedgerV3;
use support::sprint111_support::{read_fixture, run_sprint111};

#[test]
fn cumulative_safe_patch_ledger_v3_matches_fixture() {
    let bundle = run_sprint111(
        "soma_cumulative_safe_patch_ledger_v3.toml",
        "cumulative-safe-patch-ledger-v3",
    );
    let expected: CumulativeSafePatchLedgerV3 =
        read_fixture("sprint111_data/cumulative_safe_patch_ledger_v3_expected.json");
    assert_eq!(bundle.cumulative_safe_patch_ledger_v3, expected);
    assert_eq!(bundle.cumulative_safe_patch_ledger_v3.patch_count, 4);
    assert_eq!(
        bundle
            .cumulative_safe_patch_impact_report_v3
            .measured_claim_allowed,
        false
    );
}

mod support;

use soma_zero::SafeConsolidationPatchV1Config;
use support::sprint107_support::{read_fixture, run_sprint107};

#[test]
fn low_risk_candidate_selected_deterministically() {
    let bundle = run_sprint107(
        "soma_safe_consolidation_patch_selection.toml",
        "safe-consolidation-patch-selection",
    );
    let actual =
        serde_json::to_value(&bundle.safe_consolidation_patch_selection_report).expect("actual");
    let expected: serde_json::Value =
        read_fixture("sprint107_data/safe_consolidation_patch_selection_expected.json");
    assert_eq!(actual, expected);
    assert_eq!(
        bundle
            .safe_consolidation_patch_selection_report
            .selected_target_group,
        "fixture-harness-diagnostics"
    );
}

#[test]
fn no_safe_candidate_status_is_supported() {
    let mut config = SafeConsolidationPatchV1Config::default();
    config.apply_one_safe_consolidation = false;
    config.require_assertion_ledger = false;
    config.output_root = "target/sprint107-no-safe-candidate".to_string();
    let bundle = soma_zero::SafeConsolidationPatchV1Runner::default()
        .run(&config)
        .expect("run");
    assert_eq!(
        bundle
            .safe_consolidation_patch_selection_report
            .selected_status,
        "NoSafeCandidate"
    );
    assert!(
        bundle
            .safe_consolidation_patch_selection_report
            .selection_reason
            .contains("CommitteeCliSafety")
    );
}

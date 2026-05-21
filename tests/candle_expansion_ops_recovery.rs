mod support;

use soma_zero::{CandleExpansionOpsRecoveryStatus, Sprint88SevenBlockerRecoveryRunner};
use support::sprint69_support as sprint;

#[test]
fn candle_expansion_recovery_preserves_expected_coverage() {
    let config = sprint::sprint88_config_from_example(
        "soma_candle_expansion_recovery.toml",
        "candle-recovery",
    );
    let report = Sprint88SevenBlockerRecoveryRunner::default()
        .run_candle_expansion_recovery(&config)
        .expect("report");
    assert!(report.local_import_reuse_covered);
    assert!(report.missing_auth_behavior_covered);
    assert!(report.source_class_preserved);
    assert!(report.no_lookahead_preserved);
    assert!(report.storage_budget_covered);
    assert!(report.deterministic_output_covered);
    assert_eq!(
        report.recovery_status,
        CandleExpansionOpsRecoveryStatus::CandleExpansionOpsReduced
    );
}

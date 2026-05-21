mod support;

use soma_zero::{ExternalPredictionRecoveryStatus, Sprint88SevenBlockerRecoveryRunner};
use support::sprint69_support as sprint;

#[test]
fn external_prediction_recovery_preserves_research_only_contract() {
    let config = sprint::sprint88_config_from_example(
        "soma_external_prediction_recovery.toml",
        "external-recovery",
    );
    let report = Sprint88SevenBlockerRecoveryRunner::default()
        .run_external_prediction_recovery(&config)
        .expect("report");
    assert!(report.schema_v2_covered);
    assert!(report.model_card_covered);
    assert!(report.sequence_match_covered);
    assert!(report.duplicate_rejection_covered);
    assert!(report.probability_sanity_covered);
    assert!(report.account_order_secret_column_rejection_covered);
    assert!(report.research_only_promotion_covered);
    assert!(report.runtime_deferred_covered);
    assert_eq!(
        report.recovery_status,
        ExternalPredictionRecoveryStatus::ExternalPredictionReduced
    );
}

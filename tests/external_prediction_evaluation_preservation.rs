mod support;

use soma_zero::{
    ExternalPredictionEvaluationPreservationStatus, Sprint90ExternalPredictionRecoveryRunner,
};
use support::sprint69_support as sprint;

#[test]
fn external_prediction_evaluation_preservation_stays_research_only() {
    let config = sprint::sprint90_config_from_example(
        "soma_external_prediction_evaluation_preservation.toml",
        "external-evaluation-preservation",
    );
    let report = Sprint90ExternalPredictionRecoveryRunner::default()
        .run_external_prediction_evaluation_preservation(&config)
        .expect("report");
    assert_eq!(
        report.evaluation_status,
        ExternalPredictionEvaluationPreservationStatus::EvaluationPreserved
    );
    assert!(report.offline_evaluation_preserved);
    assert!(report.trinity_comparison_preserved);
    assert!(report.no_trade_comparison_preserved);
    assert!(report.risk_denied_comparison_preserved);
    assert!(report.calibration_report_preserved);
    assert!(report.risk_interaction_report_preserved);
    assert!(report.promotion_gate_research_only_preserved);
}

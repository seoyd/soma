#[path = "support/sprint69_support.rs"]
mod support;

use soma_zero::RealEvidenceModelOpsImpactStatus;

#[test]
fn model_ops_impact_requires_new_predictions_for_example() {
    let report = support::run_sprint74_bundle(
        "soma_real_modelops_impact.toml",
        "real-evidence-modelops-impact",
    )
    .real_evidence_model_ops_impact_report;
    assert_eq!(
        report.impact_status,
        RealEvidenceModelOpsImpactStatus::NeedNewPredictionCsv
    );
    assert_eq!(report.evidence_rows_added, 8);
    assert!(
        report
            .models_requiring_reevaluation
            .contains(&"ext-model-b:1.0.0".to_string())
    );
}

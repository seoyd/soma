mod support;

use soma_zero::{
    ExternalPredictionFeatureVariantReductionStatus, Sprint90ExternalPredictionRecoveryRunner,
};
use support::sprint69_support as sprint;

#[test]
fn external_prediction_feature_variant_reduction_stays_conservative() {
    let config = sprint::sprint90_config_from_example(
        "soma_external_prediction_feature_variant_reduction.toml",
        "external-feature-variants",
    );
    let report = Sprint90ExternalPredictionRecoveryRunner::default()
        .run_external_prediction_feature_variant_reduction(&config)
        .expect("report");
    assert_eq!(
        report.reduction_status,
        ExternalPredictionFeatureVariantReductionStatus::FeatureVariantReducedWithWarnings
    );
    assert!(
        report
            .repeated_variants
            .contains(&"default+test-fixtures".to_string())
    );
    assert!(
        report
            .unsafe_variants
            .contains(&"default+research-only".to_string())
    );
}

mod support;

use soma_zero::{FeatureVariantImpactProbeReportStatus, Sprint88SevenBlockerRecoveryRunner};
use support::sprint69_support as sprint;

#[test]
fn feature_variant_impact_probe_keeps_unsafe_candidates_explicit() {
    let config = sprint::sprint88_config_from_example(
        "soma_feature_variant_impact_probe.toml",
        "feature-variant-impact",
    );
    let report = Sprint88SevenBlockerRecoveryRunner::default()
        .run_feature_variant_impact_probe(&config)
        .expect("report");
    assert_eq!(
        report.report_status,
        FeatureVariantImpactProbeReportStatus::UnsafeUnificationDetected
    );
    assert!(
        report
            .repeated_feature_variants
            .contains(&"default+test-fixtures".to_string())
    );
    assert!(
        report
            .repeated_feature_variants
            .contains(&"default+research-only".to_string())
    );
    assert!(
        report
            .impacted_families
            .contains(&"ExternalPrediction".to_string())
    );
    assert!(!report.unsafe_unification_candidates.is_empty());
}

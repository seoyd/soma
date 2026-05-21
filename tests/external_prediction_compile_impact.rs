mod support;

use soma_zero::{ExternalPredictionCompileImpactStatus, Sprint90ExternalPredictionRecoveryRunner};
use support::sprint69_support as sprint;

#[test]
fn external_prediction_compile_impact_stays_sample_backed_without_fake_timings() {
    let config = sprint::sprint90_config_from_example(
        "soma_external_prediction_compile_impact.toml",
        "external-compile-impact",
    );
    let report = Sprint90ExternalPredictionRecoveryRunner::default()
        .run_external_prediction_compile_impact(&config)
        .expect("report");
    assert_eq!(
        report.impact_status,
        ExternalPredictionCompileImpactStatus::CompileImpactSampleBacked
    );
    assert_eq!(report.target_count_before, Some(6));
    assert_eq!(report.target_count_after, Some(5));
    assert_eq!(report.external_family_delta, Some(1));
    assert!(!report.measured);
    assert!(report.sample_backed);
    assert!(report.blocked_targets.contains(&"KrxEvidence".to_string()));
}

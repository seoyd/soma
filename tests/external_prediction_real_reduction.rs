mod support;

use soma_zero::{
    CompileFamilyV2, ExternalPredictionRealReductionAction, ExternalPredictionRealReductionConfig,
    ExternalPredictionRealReductionStatus, Sprint90ExternalPredictionRecoveryRunner,
};
use support::sprint69_support as sprint;

#[test]
fn external_prediction_config_defaults_stay_conservative() {
    let config = ExternalPredictionRealReductionConfig::default();
    assert_eq!(config.target_family, CompileFamilyV2::ExternalPrediction);
    assert!(config.preserve_assertions);
    assert!(config.preserve_safety_guards);
    assert!(config.preserve_prediction_schema_checks);
    assert!(config.preserve_model_card_checks);
    assert!(config.preserve_duplicate_rejection);
    assert!(config.preserve_probability_sanity);
    assert!(config.preserve_forbidden_column_rejection);
    assert!(config.preserve_runtime_deferred_checks);
}

#[test]
fn external_prediction_config_rejects_remote_paths() {
    let config = ExternalPredictionRealReductionConfig {
        sprint89_bundle_paths: vec!["https://example.com/summary.json".to_string()],
        ..ExternalPredictionRealReductionConfig::default()
    };
    assert!(config.validate().is_err());
}

#[test]
fn external_prediction_plan_keeps_donor_lineage_and_actions() {
    let config = sprint::sprint90_config_from_example(
        "soma_external_prediction_real_reduction_plan.toml",
        "external-real-plan",
    );
    let plan = Sprint90ExternalPredictionRecoveryRunner::default()
        .run_external_prediction_real_reduction_plan(&config)
        .expect("plan");
    assert!(
        plan.target_files
            .contains(&"tests/external_prediction_family_suite.rs".to_string())
    );
    assert!(
        plan.donor_files
            .contains(&"tests/external_prediction_import_v2.rs".to_string())
    );
    assert!(
        plan.actions
            .contains(&ExternalPredictionRealReductionAction::VerifyGroupedSuiteCoverage)
    );
    assert!(
        plan.actions
            .contains(&ExternalPredictionRealReductionAction::ApplySharedFixtureHarness)
    );
    assert!(plan.actions.contains(
        &ExternalPredictionRealReductionAction::ReducePredictionSchemaFixtureDuplication
    ));
}

#[test]
fn external_prediction_bundle_marks_family_as_reduced() {
    let bundle = sprint::run_sprint90_bundle(
        "soma_sprint90_external_prediction_recover.toml",
        "external-real-reduction-bundle",
    );
    assert_eq!(
        bundle
            .external_prediction_real_reduction_report
            .reduction_status,
        ExternalPredictionRealReductionStatus::ExternalPredictionRealReduced
    );
    assert_eq!(
        bundle
            .seven_blocker_queue_progress_report_v6
            .primary_next_family,
        "KrxEvidence"
    );
}

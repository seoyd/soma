mod support;

use soma_zero::{
    CandleExpansionRealReductionAction, CandleExpansionRealReductionConfig,
    CandleExpansionRealReductionStatus, CompileFamilyV2, Sprint89CandleRecoveryRunner,
};
use support::sprint69_support as sprint;

#[test]
fn candle_reduction_config_defaults_stay_conservative() {
    let config = CandleExpansionRealReductionConfig::default();
    assert_eq!(config.target_family, CompileFamilyV2::CandleExpansionOps);
    assert!(config.preserve_assertions);
    assert!(config.preserve_safety_guards);
    assert!(config.preserve_source_boundary);
    assert!(config.preserve_no_lookahead);
    assert!(config.preserve_storage_budget_checks);
    assert!(config.preserve_missing_auth_checks);
}

#[test]
fn candle_reduction_config_rejects_remote_paths() {
    let config = CandleExpansionRealReductionConfig {
        sprint88_bundle_paths: vec!["https://example.com/summary.json".to_string()],
        ..CandleExpansionRealReductionConfig::default()
    };
    assert!(config.validate().is_err());
}

#[test]
fn candle_real_reduction_plan_keeps_donor_lineage_and_actions() {
    let config = sprint::sprint89_config_from_example(
        "soma_candle_real_reduction_plan.toml",
        "candle-real-plan",
    );
    let plan = Sprint89CandleRecoveryRunner::default()
        .run_candle_real_reduction_plan(&config)
        .expect("plan");
    assert!(
        plan.donor_files
            .contains(&"tests/official_candle_expansion_runner.rs".to_string())
    );
    assert!(
        plan.target_files
            .contains(&"tests/candle_expansion_ops_suite.rs".to_string())
    );
    assert!(
        plan.actions
            .contains(&CandleExpansionRealReductionAction::VerifyGroupedSuiteCoverage)
    );
    assert!(
        plan.actions
            .contains(&CandleExpansionRealReductionAction::ApplySharedFixtureHarness)
    );
}

#[test]
fn candle_real_reduction_bundle_marks_candle_as_reduced() {
    let bundle = sprint::run_sprint89_bundle(
        "soma_sprint89_candle_recover.toml",
        "candle-real-reduction-bundle",
    );
    assert_eq!(
        bundle
            .candle_expansion_real_reduction_report
            .reduction_status,
        CandleExpansionRealReductionStatus::CandleExpansionOpsRealReduced
    );
    assert_eq!(
        bundle
            .seven_blocker_queue_progress_report_v5
            .primary_next_family,
        "ExternalPrediction"
    );
}

mod support;

use std::fs;

use soma_zero::DeferredRealObservationExecutionConfig;
use support::sprint117_support::run_sprint117;

#[test]
fn deferred_real_observation_execution_config_and_bundle_are_sane() {
    let config = DeferredRealObservationExecutionConfig::default();
    assert!(config.require_actual_vs_carried_forward_separation);
    assert!(config.require_timeout_cleanup_actual_counts);
    assert!(config.require_cargo_json_actual_parsing);
    assert!(config.require_acceptance_truth_gate);
    assert!(config.require_consolidation_still_paused);
    assert!(!config.allow_fifth_patch_application);
    assert!(!config.allow_assertion_movement);
    assert!(!config.allow_test_target_retirement);
    let toml = config.to_toml_string().expect("toml");
    for forbidden in [
        "training_enabled",
        "broker",
        "order",
        "account",
        "runtime_field",
    ] {
        assert!(!toml.contains(forbidden));
    }
    let invalid = DeferredRealObservationExecutionConfig {
        output_root: "https://example.invalid/out".to_string(),
        ..config.clone()
    };
    assert!(invalid.validate().is_err());
    let bundle = run_sprint117(
        "soma_sprint117_deferred_real_observation.toml",
        "deferred-real-observation-execution",
    );
    assert_eq!(bundle.storage_report.file_count, 39);
    assert_eq!(
        bundle
            .deferred_observation_selection_report_v1
            .selected_observations,
        vec!["RealNoRun", "RealFullWorkspace", "RealCargoJson"]
    );
    assert_eq!(
        bundle
            .deferred_observation_execution_plan_v1
            .execution_order,
        vec!["RealCargoJson", "RealNoRun", "RealFullWorkspace"]
    );
    assert_eq!(
        bundle
            .observation_backlog_completion_report_v2
            .remaining_count,
        3
    );
    let out = std::path::PathBuf::from(&bundle.storage_report.output_dir);
    assert!(out.join("summary.txt").exists());
    assert!(out.join("storage_report.txt").exists());
    let summary = fs::read_to_string(out.join("summary.txt")).expect("summary");
    assert!(summary.contains("## 1. Sprint summary"));
    assert!(summary.contains("## 57. Next gstack sprint recommendation"));
    assert!(summary.contains("## 33. Acceptance truth gate v18"));
}

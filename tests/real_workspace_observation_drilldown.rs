mod support;

use std::fs;

use soma_zero::RealWorkspaceObservationDrilldownConfig;
use support::sprint113_support::run_sprint113;

#[test]
fn real_workspace_observation_drilldown_defaults_and_bundle_hold() {
    let config = RealWorkspaceObservationDrilldownConfig::default();
    assert!(!config.run_real_no_run_observation);
    assert!(!config.run_real_full_observation);
    assert!(!config.run_real_cargo_json_capture);
    assert!(!config.run_nextest_probe);
    assert!(!config.run_nextest_partition_probe);
    assert!(!config.run_sccache_probe);
    assert!(!config.run_sccache_local_pilot);
    assert!(config.require_actual_observation_preservation);
    assert!(config.require_timeout_cleanup_actual_counts);
    assert!(config.require_cargo_json_actual_parsing);
    assert!(config.require_fifth_patch_gate);
    assert!(!config.allow_fifth_patch_application);
    let toml = config.to_toml_string().expect("toml");
    assert!(!toml.contains("live_inference"));
    assert!(!toml.contains("model_training"));
    assert!(!toml.contains("broker"));
    let mut invalid = config.clone();
    invalid.output_root = "https://example.invalid/out".to_string();
    assert!(invalid.validate().is_err());

    let bundle = run_sprint113(
        "soma_sprint113_real_workspace_observation.toml",
        "real-workspace-observation-drilldown",
    );
    assert_eq!(bundle.storage_report.file_count, 48);
    assert!(
        bundle
            .storage_report
            .written_files
            .contains(&"storage_report.txt".to_string())
    );
    assert!(
        bundle
            .storage_report
            .written_files
            .contains(&"summary.txt".to_string())
    );
    assert!(
        !bundle
            .fifth_patch_decision_gate_v3
            .fifth_patch_applied_this_sprint
    );
    let summary_path = std::path::Path::new(&bundle.storage_report.output_dir).join("summary.txt");
    let summary = fs::read_to_string(summary_path).expect("summary");
    assert!(summary.contains("## 57. Next gstack sprint recommendation"));
    assert!(summary.contains("- file_count=48"));
}

mod support;

use std::fs;

use soma_zero::WorkspaceDiagnosticPilotV1Config;
use support::sprint112_support::run_sprint112;

#[test]
fn workspace_diagnostic_pilot_config_defaults_and_bundle_hold() {
    let config = WorkspaceDiagnosticPilotV1Config::default();
    assert!(!config.run_nextest_probe);
    assert!(!config.run_sccache_probe);
    assert!(!config.run_real_no_run_observation);
    assert!(!config.run_real_full_observation);
    assert!(!config.run_cargo_json_progress_capture);
    assert!(!config.run_cargo_check_timing);
    assert!(!config.run_cargo_build_timing);
    assert!(config.require_nextest_sccache_diagnostic);
    assert!(config.require_acceptance_truth_gate);
    assert!(config.require_fifth_patch_re_evaluation);
    assert!(!config.allow_fifth_patch_application);
    let toml = config.to_toml_string().expect("toml");
    assert!(!toml.contains("live_inference"));
    assert!(!toml.contains("model_training"));
    assert!(!toml.contains("broker"));
    let mut invalid = config.clone();
    invalid.output_root = "https://example.invalid/out".to_string();
    assert!(invalid.validate().is_err());
    let bundle = run_sprint112(
        "soma_sprint112_workspace_diagnostic_pilot.toml",
        "workspace-diagnostic-pilot",
    );
    assert_eq!(
        bundle
            .final_summary
            .lines()
            .filter(|line| line.starts_with("## "))
            .count(),
        54
    );
    assert_eq!(
        bundle
            .fifth_patch_decision_gate_v2
            .fifth_patch_applied_this_sprint,
        false
    );
    assert_eq!(bundle.storage_report.file_count, 47);
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
    let summary_path = std::path::Path::new(&bundle.storage_report.output_dir).join("summary.txt");
    let summary = fs::read_to_string(summary_path).expect("summary");
    assert!(summary.contains("- file_count=47."));
}

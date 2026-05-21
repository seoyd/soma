mod support;

use std::fs;

use soma_zero::MixedFamilyIsolationV1Config;
use support::sprint114_support::run_sprint114;

#[test]
fn mixed_family_isolation_config_and_bundle_are_sane() {
    let config = MixedFamilyIsolationV1Config::default();
    assert!(config.require_assertion_migration_feasibility);
    assert!(config.require_equivalent_coverage_drilldown);
    assert!(config.require_fifth_patch_gate_v4);
    assert!(!config.allow_fifth_patch_application);
    assert!(!config.run_real_no_run_observation);
    assert!(!config.run_real_full_observation);
    assert!(!config.run_cargo_json_suspect_trace);
    assert!(!config.run_suspect_target_rustc_probe);
    let toml = config.to_toml_string().expect("toml");
    for forbidden in ["training", "broker", "order", "account", "runtime_field"] {
        assert!(!toml.contains(forbidden), "unexpected field {forbidden}");
    }

    let invalid = MixedFamilyIsolationV1Config {
        output_root: "https://example.invalid/out".to_string(),
        ..config.clone()
    };
    assert!(invalid.validate().is_err());
    let invalid_scheme_without_slashes = MixedFamilyIsolationV1Config {
        output_root: "http:example.invalid/out".to_string(),
        ..config.clone()
    };
    assert!(invalid_scheme_without_slashes.validate().is_err());

    let bundle = run_sprint114(
        "soma_sprint114_mixed_family_isolation.toml",
        "mixed-family-isolation-v1",
    );
    assert_eq!(bundle.storage_report.file_count, 47);
    assert!(
        !bundle
            .fifth_patch_no_apply_guarantee_report_v3
            .fifth_patch_applied
    );
    assert!(
        bundle
            .fifth_patch_no_apply_guarantee_report_v3
            .retired_files
            .is_empty()
    );
    assert!(
        bundle
            .fifth_patch_no_apply_guarantee_report_v3
            .moved_assertions
            .is_empty()
    );

    let out = std::path::PathBuf::from(&bundle.storage_report.output_dir);
    assert!(out.join("summary.txt").exists());
    assert!(out.join("storage_report.txt").exists());
    let summary = fs::read_to_string(out.join("summary.txt")).expect("summary");
    assert!(summary.contains("## 59. Next gstack sprint recommendation"));
    assert!(summary.contains("file_count=47"));
    assert_eq!(fs::read_dir(out).expect("list output").count(), 47);
}

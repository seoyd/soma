mod support;

use std::fs;

use soma_zero::ConsolidationStopResumeGovernanceConfig;
use support::sprint115_support::run_sprint115;

#[test]
fn consolidation_stop_resume_governance_config_and_bundle_are_sane() {
    let config = ConsolidationStopResumeGovernanceConfig::default();
    assert!(config.require_stop_resume_decision);
    assert!(config.require_assertion_destination_proof);
    assert!(config.require_evidence_blur_gate);
    assert!(config.require_track_split);
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
        assert!(!toml.contains(forbidden), "unexpected field {forbidden}");
    }

    let invalid = ConsolidationStopResumeGovernanceConfig {
        output_root: "https://example.invalid/out".to_string(),
        ..config.clone()
    };
    assert!(invalid.validate().is_err());
    let invalid_scheme_without_slashes = ConsolidationStopResumeGovernanceConfig {
        output_root: "http:example.invalid/out".to_string(),
        ..config.clone()
    };
    assert!(invalid_scheme_without_slashes.validate().is_err());

    let bundle = run_sprint115(
        "soma_sprint115_consolidation_governance.toml",
        "consolidation-stop-resume-governance",
    );
    assert_eq!(bundle.storage_report.file_count, 49);
    assert!(
        bundle
            .consolidation_stop_decision_report_v1
            .stop_recommended
    );
    assert_eq!(
        bundle.consolidation_resume_decision_report_v1.resume_status,
        "ConsolidationResumeNeedsProof"
    );
    assert!(
        !bundle
            .fifth_patch_no_apply_guarantee_report_v4
            .fifth_patch_applied
    );
    assert!(
        bundle
            .fifth_patch_no_apply_guarantee_report_v4
            .retired_files
            .is_empty()
    );
    assert!(
        bundle
            .fifth_patch_no_apply_guarantee_report_v4
            .moved_assertions
            .is_empty()
    );

    let out = std::path::PathBuf::from(&bundle.storage_report.output_dir);
    assert!(out.join("summary.txt").exists());
    assert!(out.join("storage_report.txt").exists());
    let summary = fs::read_to_string(out.join("summary.txt")).expect("summary");
    assert!(summary.contains("## 2. Why Sprint 115 was needed"));
    assert!(summary.contains("## 60. Next gstack sprint recommendation"));
    assert!(summary.contains("file_count=49"));
    assert_eq!(fs::read_dir(out).expect("list output").count(), 49);
}

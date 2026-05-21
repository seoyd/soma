mod support;

use soma_zero::PaperRotationWarningClosureConfig;
use support::sprint103_support::run_sprint103;

#[test]
fn sprint103_config_and_bundle_cover_required_guards() {
    let config = PaperRotationWarningClosureConfig::default();
    assert!(config.require_rotation_warning_closure);
    assert!(config.require_lower_confidence_closure);
    assert!(config.preserve_paper_only);
    assert!(config.preserve_no_live_activation);
    assert!(config.preserve_runtime_deferred);
    assert!(config.validate().is_ok());

    let mut invalid = config.clone();
    invalid.output_root = "https://example.com/out".to_string();
    assert!(invalid.validate().is_err());
    invalid = config.clone();
    invalid.max_replay_scenarios = 0;
    assert!(invalid.validate().is_err());

    let toml = config.to_toml_string().expect("toml");
    for needle in ["broker", "order", "account"] {
        assert!(!toml.contains(needle), "unexpected field {needle}");
    }

    let bundle = run_sprint103(
        "soma_sprint103_paper_rotation_close.toml",
        "sprint103-bundle",
    );
    assert!(bundle.final_summary.contains("## 1. Sprint summary"));
    assert!(
        bundle
            .final_summary
            .contains("## 49. Next gstack sprint recommendation")
    );
    assert!(bundle.storage_report.file_count >= 38);
    assert!(
        bundle
            .safety_coverage_preservation_report_v19
            .paper_rotation_not_order_execution_guard_present
    );
    assert!(
        bundle
            .safety_coverage_preservation_report_v19
            .no_silent_confidence_upgrade_guard_present
    );
    assert!(
        !bundle
            .paper_rotation_readiness_gate_v2
            .live_rotation_allowed
    );
    assert!(std::path::Path::new(&bundle.storage_report.output_dir).exists());
}

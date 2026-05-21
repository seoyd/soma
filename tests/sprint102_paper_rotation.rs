mod support;

use soma_zero::EighteenArchetypePaperRotationConfig;
use support::sprint102_support::run_sprint102;

#[test]
fn sprint102_config_and_bundle_cover_required_guards() {
    let config = EighteenArchetypePaperRotationConfig::default();
    assert!(config.require_paper_only);
    assert!(config.require_no_live_activation);
    assert!(config.require_source_confidence_weighting);
    assert!(config.require_lower_confidence_review);
    assert!(config.preserve_runtime_deferred);
    assert!(config.validate().is_ok());

    let mut invalid = config.clone();
    invalid.output_root = "https://example.com/out".to_string();
    assert!(invalid.validate().is_err());
    invalid = config.clone();
    invalid.max_scenarios = 0;
    assert!(invalid.validate().is_err());

    let toml = config.to_toml_string().expect("toml");
    for needle in ["broker", "order", "account"] {
        assert!(!toml.contains(needle), "unexpected field {needle}");
    }

    let bundle = run_sprint102("soma_sprint102_paper_rotation.toml", "sprint102-bundle");
    assert!(bundle.final_summary.contains("## 1. Sprint summary"));
    assert!(
        bundle
            .final_summary
            .contains("## 48. Next gstack sprint recommendation")
    );
    assert!(bundle.storage_report.file_count >= 32);
    assert!(
        bundle
            .safety_coverage_preservation_report_v18
            .live_trading_guard_present
    );
    assert!(
        bundle
            .safety_coverage_preservation_report_v18
            .broker_guard_present
    );
    assert!(
        bundle
            .safety_coverage_preservation_report_v18
            .order_guard_present
    );
    assert!(
        bundle
            .safety_coverage_preservation_report_v18
            .account_guard_present
    );
    assert!(
        bundle
            .safety_coverage_preservation_report_v18
            .runtime_llm_guard_present
    );
    assert!(
        bundle
            .safety_coverage_preservation_report_v18
            .browser_execution_guard_present
    );
    assert!(
        bundle
            .safety_coverage_preservation_report_v18
            .investor_impersonation_guard_present
    );
    assert!(
        bundle
            .safety_coverage_preservation_report_v18
            .do_not_learn_guard_present
    );
    assert!(
        bundle
            .safety_coverage_preservation_report_v18
            .paper_rotation_not_order_execution_guard_present
    );

    assert!(std::path::Path::new(&bundle.storage_report.output_dir).exists());
}

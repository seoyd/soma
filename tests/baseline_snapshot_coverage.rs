mod common;
#[path = "support/sprint69_support.rs"]
mod sprint69_support;

use soma_zero::{
    BaselineSnapshotCoverageConfig, BaselineSnapshotCoverageRunner, BaselineSnapshotCoverageStatus,
    ComparisonTargetKind, MissingComparisonTargetReportStatus,
};

#[test]
fn baseline_snapshot_coverage_config_defaults_and_remote_paths_are_guarded() {
    let config = BaselineSnapshotCoverageConfig::default();
    assert!(config.require_baseline_for_comparable_versions);
    assert!(config.require_current_snapshot);
    assert!(config.allow_current_only_diagnostic);
    let encoded = toml::to_string(&config).expect("serialize config");
    for forbidden in [
        "broker_",
        "order_",
        "account_",
        "live_",
        "runtime_",
        "training_",
    ] {
        assert!(
            !encoded.contains(forbidden),
            "unexpected forbidden config field: {forbidden}"
        );
    }

    let mut bad = config.clone();
    bad.model_ops_trace_bundle_paths = vec!["https://example.com/trace.json".to_string()];
    assert!(bad.validate().is_err());
}

#[test]
fn baseline_snapshot_coverage_limits_are_enforced() {
    let mut config = sprint69_support::coverage_config_from_example(
        "soma_baseline_snapshot_coverage.toml",
        "coverage-limits-models",
    );
    config.max_models = 1;
    assert!(
        BaselineSnapshotCoverageRunner::default()
            .run(&config)
            .is_err()
    );

    let mut config = sprint69_support::coverage_config_from_example(
        "soma_baseline_snapshot_coverage.toml",
        "coverage-limits-versions",
    );
    config.max_versions = 1;
    assert!(
        BaselineSnapshotCoverageRunner::default()
            .run(&config)
            .is_err()
    );

    let mut config = sprint69_support::coverage_config_from_example(
        "soma_baseline_snapshot_coverage.toml",
        "coverage-limits-snapshots",
    );
    config.max_snapshots = 1;
    assert!(
        BaselineSnapshotCoverageRunner::default()
            .run(&config)
            .is_err()
    );

    let mut config = sprint69_support::coverage_config_from_example(
        "soma_baseline_snapshot_coverage.toml",
        "coverage-limits-bytes",
    );
    config.max_bytes = 64;
    assert!(
        BaselineSnapshotCoverageRunner::default()
            .run(&config)
            .is_err()
    );
}

#[test]
fn baseline_snapshot_coverage_closes_missing_target_gap() {
    let bundle =
        sprint69_support::run_coverage("soma_baseline_snapshot_coverage.toml", "coverage-ready");
    assert_eq!(
        bundle.baseline_snapshot_coverage_report.coverage_status,
        BaselineSnapshotCoverageStatus::CoverageReady
    );
    assert_eq!(bundle.comparison_target_registry.comparable_count, 4);
    assert_eq!(bundle.comparison_target_registry.missing_count, 0);
    assert_eq!(
        bundle.missing_comparison_target_report.report_status,
        MissingComparisonTargetReportStatus::TargetsResolved
    );

    let ext_model_a = bundle
        .comparison_target_registry
        .records
        .iter()
        .find(|item| item.model_id == "ext-model-a" && item.model_version == "1.2.0")
        .expect("registry record");
    assert_eq!(
        ext_model_a.comparison_target_kind,
        ComparisonTargetKind::Baseline
    );
    assert!(ext_model_a.comparable);

    let coverage_item = bundle
        .baseline_snapshot_coverage_report
        .items
        .iter()
        .find(|item| item.model_id == "ext-model-a" && item.model_version == "1.2.0")
        .expect("coverage item");
    assert!(coverage_item.has_baseline_snapshot);
    assert!(coverage_item.has_current_snapshot);
    assert!(coverage_item.comparison_target_available);
}

#[test]
fn current_only_diagnostic_and_same_version_current_registry_modes_are_explicit() {
    let mut config = sprint69_support::coverage_config_from_example(
        "soma_baseline_snapshot_coverage.toml",
        "coverage-current-only",
    );
    config.baseline_snapshot_paths.clear();
    config.current_snapshot_paths = vec![
        sprint69_support::example_path("sprint69_data/current_snapshot_ext_model_a_1_2_0.json")
            .display()
            .to_string(),
    ];
    let registry = BaselineSnapshotCoverageRunner::default()
        .run_comparison_target_registry(&config)
        .expect("run registry");
    let record = registry
        .records
        .iter()
        .find(|item| item.model_id == "ext-model-a" && item.model_version == "1.2.0")
        .expect("current-only record");
    assert_eq!(
        record.comparison_target_kind,
        ComparisonTargetKind::CurrentOnlyDiagnostic
    );
    assert!(record.diagnostic_only);

    let mut strict = config.clone();
    strict.allow_current_only_diagnostic = false;
    let registry = BaselineSnapshotCoverageRunner::default()
        .run_comparison_target_registry(&strict)
        .expect("run strict registry");
    let record = registry
        .records
        .iter()
        .find(|item| item.model_id == "ext-model-a" && item.model_version == "1.2.0")
        .expect("same-version-current record");
    assert_eq!(
        record.comparison_target_kind,
        ComparisonTargetKind::SameVersionCurrent
    );
    assert!(!record.comparable);
    assert!(!record.diagnostic_only);
}

#[test]
fn missing_target_resolution_matches_expected_fixture() {
    let bundle = sprint69_support::run_coverage(
        "soma_baseline_snapshot_coverage.toml",
        "coverage-resolution",
    );
    let expected: serde_json::Value = sprint69_support::read_json(sprint69_support::example_path(
        "sprint69_data/expected_missing_target_resolution.json",
    ));
    let actual = serde_json::to_value(&bundle.missing_comparison_target_resolution_report)
        .expect("to value");
    assert_eq!(actual["resolution_status"], expected["resolution_status"]);
    assert_eq!(actual["resolved_items"], expected["resolved_items"]);
    assert_eq!(actual["unresolved_items"], expected["unresolved_items"]);
    assert_eq!(actual["current_only_items"], expected["current_only_items"]);
    assert_eq!(
        actual["added_baseline_refs"].as_array().map(Vec::len),
        expected["added_baseline_refs"].as_array().map(Vec::len)
    );
    assert_eq!(
        actual["added_current_refs"].as_array().map(Vec::len),
        expected["added_current_refs"].as_array().map(Vec::len)
    );
}

mod support;

use serde_json::to_value;
use soma_zero::{WorkspaceTimeoutReductionQueueConfig, WorkspaceTimeoutReductionQueueV1};
use support::sprint118_support::{read_fixture, run_sprint118};

#[test]
fn workspace_timeout_reduction_queue_matches_expected_and_config_is_safe() {
    let bundle = run_sprint118(
        "soma_workspace_timeout_reduction_queue_v1.toml",
        "workspace-timeout-reduction-queue",
    );
    let expected: WorkspaceTimeoutReductionQueueV1 =
        read_fixture("sprint118_data/timeout_reduction_queue_expected.json");
    assert_eq!(bundle.workspace_timeout_reduction_queue_v1, expected);
    assert!(bundle.final_summary.contains("## 1. Sprint summary"));
    assert!(
        bundle
            .final_summary
            .contains("## 44. Acceptance truth gate v19")
    );
    assert!(
        bundle
            .final_summary
            .contains("## 67. Next gstack sprint recommendation")
    );
    assert_eq!(bundle.final_summary.matches("\n## ").count() + 1, 67);
    let config = WorkspaceTimeoutReductionQueueConfig::default();
    assert!(config.validate().is_ok());
    assert!(config.require_cargo_json_reason_analysis);
    assert!(config.require_timeout_reduction_queue);
    assert!(config.require_acceptance_truth_gate);
    assert!(!config.allow_fifth_patch_application);
    assert!(!config.allow_assertion_movement);
    assert!(!config.allow_test_target_retirement);
    let remote = WorkspaceTimeoutReductionQueueConfig {
        sprint117_truth_paths: Some(vec!["https://example.invalid/config.json".to_string()]),
        ..config.clone()
    };
    assert!(remote.validate().is_err());
    let json = to_value(config).expect("serialize config");
    let text = json.to_string();
    for forbidden in [
        "runtime_enabled",
        "training_enabled",
        "broker",
        "order",
        "account",
    ] {
        assert!(
            !text.contains(forbidden),
            "forbidden field leaked: {forbidden}"
        );
    }
}

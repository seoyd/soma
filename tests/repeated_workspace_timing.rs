#[path = "support/sprint69_support.rs"]
mod support;

use soma_zero::{
    RepeatedCommandTiming, RepeatedWorkspaceTimingConfig, RepeatedWorkspaceTimingStatus,
    TimingRunCondition, build_repeated_workspace_timing_report,
};

#[test]
fn repeated_workspace_timing_config_defaults_and_guards_work() {
    let config = RepeatedWorkspaceTimingConfig::from_toml_path(&support::example_path(
        "soma_repeated_workspace_timing.toml",
    ))
    .expect("parse repeated timing config");
    assert!(!config.run_real_commands);
    assert!(config.repetitions > 0);
    assert!(config.warmup_runs < config.repetitions);
    let json = serde_json::to_string(&config).expect("serialize config");
    assert!(!json.contains("broker"));
    assert!(!json.contains("account"));
    assert!(!json.contains("live"));

    let mut remote = RepeatedWorkspaceTimingConfig::default();
    remote.output_root = "https://example.com/out".to_string();
    assert!(remote.validate().is_err());

    let mut bad_repetitions = RepeatedWorkspaceTimingConfig::default();
    bad_repetitions.repetitions = 0;
    assert!(bad_repetitions.validate().is_err());

    let mut bad_warmup = RepeatedWorkspaceTimingConfig::default();
    bad_warmup.warmup_runs = bad_warmup.repetitions;
    assert!(bad_warmup.validate().is_err());
}

#[test]
fn repeated_workspace_timing_report_computes_sample_and_failure_states() {
    let sample = build_repeated_workspace_timing_report(
        "sample",
        vec![
            RepeatedCommandTiming {
                command: "cargo check --workspace".to_string(),
                condition: TimingRunCondition::SampleBacked,
                run_index: 0,
                wall_time_ms: Some(100),
                exit_success: Some(true),
                stdout_summary: None,
                stderr_summary: None,
                reason_codes: Vec::new(),
            },
            RepeatedCommandTiming {
                command: "cargo check --workspace".to_string(),
                condition: TimingRunCondition::SampleBacked,
                run_index: 1,
                wall_time_ms: Some(300),
                exit_success: Some(true),
                stdout_summary: None,
                stderr_summary: None,
                reason_codes: Vec::new(),
            },
        ],
        None,
        None,
    );
    assert_eq!(
        sample.timing_status,
        RepeatedWorkspaceTimingStatus::SampleBacked
    );
    assert_eq!(sample.aggregate_by_command[0].median_ms, Some(300));
    assert_eq!(sample.aggregate_by_command[0].p95_ms, Some(300));

    let failed = build_repeated_workspace_timing_report(
        "failed",
        vec![RepeatedCommandTiming {
            command: "cargo test --workspace --quiet".to_string(),
            condition: TimingRunCondition::Warm,
            run_index: 0,
            wall_time_ms: Some(1000),
            exit_success: Some(false),
            stdout_summary: None,
            stderr_summary: Some("failed".to_string()),
            reason_codes: Vec::new(),
        }],
        None,
        None,
    );
    assert_eq!(failed.timing_status, RepeatedWorkspaceTimingStatus::Failed);
}

#[test]
fn repeated_workspace_timing_example_is_sample_backed() {
    let bundle = support::run_sprint77_bundle(
        "soma_repeated_workspace_timing.toml",
        "repeated-workspace-timing",
    );
    assert_eq!(
        bundle.repeated_workspace_timing_report.timing_status,
        RepeatedWorkspaceTimingStatus::SampleBacked
    );
}

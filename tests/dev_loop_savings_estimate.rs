use soma_zero::{
    DevLoopSavingsEstimateStatus, NextestPilotConfig, NextestPilotStatus, SccachePilotConfig,
    SccachePilotStatus, TimingRunCondition, build_dev_loop_savings_estimate,
    build_nextest_pilot_report, build_repeated_workspace_timing_report, build_sccache_pilot_report,
};

#[test]
fn dev_loop_savings_estimate_handles_measured_and_missing_timing() {
    let measured = build_dev_loop_savings_estimate(
        "savings",
        &build_repeated_workspace_timing_report(
            "timing",
            vec![
                soma_zero::RepeatedCommandTiming {
                    command: "cargo check --workspace".to_string(),
                    condition: TimingRunCondition::Warm,
                    run_index: 0,
                    wall_time_ms: Some(20000),
                    exit_success: Some(true),
                    stdout_summary: None,
                    stderr_summary: None,
                    reason_codes: Vec::new(),
                },
                soma_zero::RepeatedCommandTiming {
                    command: "representative CLI smoke".to_string(),
                    condition: TimingRunCondition::Warm,
                    run_index: 0,
                    wall_time_ms: Some(12000),
                    exit_success: Some(true),
                    stdout_summary: None,
                    stderr_summary: None,
                    reason_codes: Vec::new(),
                },
                soma_zero::RepeatedCommandTiming {
                    command: "full acceptance loop".to_string(),
                    condition: TimingRunCondition::Warm,
                    run_index: 0,
                    wall_time_ms: Some(200000),
                    exit_success: Some(true),
                    stdout_summary: None,
                    stderr_summary: None,
                    reason_codes: Vec::new(),
                },
            ],
            None,
            None,
        ),
    );
    assert!(matches!(
        measured.estimate_status,
        DevLoopSavingsEstimateStatus::SavingsEstimated
            | DevLoopSavingsEstimateStatus::SavingsEstimatedWithLowConfidence
    ));

    let missing = build_dev_loop_savings_estimate(
        "missing",
        &build_repeated_workspace_timing_report("missing", Vec::new(), None, None),
    );
    assert_eq!(
        missing.estimate_status,
        DevLoopSavingsEstimateStatus::TimingMissing
    );
}

#[test]
fn nextest_and_sccache_pilot_reports_cover_available_and_unavailable_states() {
    let nextest_unavailable = build_nextest_pilot_report(&NextestPilotConfig {
        pilot_id: "nextest".to_string(),
        nextest_available: false,
        run_nextest_pilot: false,
        sample_commands: vec!["cargo nextest run --workspace --status-level fail".to_string()],
        output_root: "target/sprint77".to_string(),
        reason_codes: Vec::new(),
    });
    assert_eq!(
        nextest_unavailable.pilot_status,
        NextestPilotStatus::NextestUnavailable
    );

    let nextest_available = build_nextest_pilot_report(&NextestPilotConfig {
        pilot_id: "nextest".to_string(),
        nextest_available: true,
        run_nextest_pilot: true,
        sample_commands: vec!["cargo nextest run --workspace --status-level fail".to_string()],
        output_root: "target/sprint77".to_string(),
        reason_codes: Vec::new(),
    });
    assert!(matches!(
        nextest_available.pilot_status,
        NextestPilotStatus::NextestPilotUseful
    ));

    let sccache_unavailable = build_sccache_pilot_report(&SccachePilotConfig {
        pilot_id: "sccache".to_string(),
        sccache_available: false,
        run_sccache_pilot: false,
        local_cache_only: true,
        output_root: "target/sprint77".to_string(),
        reason_codes: Vec::new(),
    });
    assert_eq!(
        sccache_unavailable.pilot_status,
        SccachePilotStatus::SccacheUnavailable
    );

    let sccache_available = build_sccache_pilot_report(&SccachePilotConfig {
        pilot_id: "sccache".to_string(),
        sccache_available: true,
        run_sccache_pilot: true,
        local_cache_only: true,
        output_root: "target/sprint77".to_string(),
        reason_codes: Vec::new(),
    });
    assert!(sccache_available.local_cache_only);
    assert_eq!(
        sccache_available.pilot_status,
        SccachePilotStatus::SccachePilotUseful
    );
}

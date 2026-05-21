use soma_zero::{
    TestTierKind, TestTierStatus, build_test_tier_config, build_test_tier_runner_report,
};

#[test]
fn test_tier_config_contains_expected_commands() {
    let config = build_test_tier_config("tier-config");
    assert!(
        config
            .quick_commands
            .iter()
            .any(|command| command.command == "cargo check --workspace")
    );
    assert!(
        config
            .sprint_commands
            .iter()
            .any(|command| command.command == "cargo fmt --all")
    );
    assert!(config.full_commands.iter().any(|command| command.command
        == "cargo test --workspace --quiet"
        && command.required_for_ship));
    assert!(
        config
            .smoke_commands
            .iter()
            .any(|command| command.command.contains("toolchain-version-report"))
    );
    assert!(
        config
            .audit_commands
            .iter()
            .any(|command| command.purpose.contains("broker/order/account absence"))
    );
    assert_eq!(config, build_test_tier_config("tier-config"));
}

#[test]
fn test_tier_runner_statuses_cover_pass_fail_and_not_run() {
    let passed = build_test_tier_runner_report(
        "quick-pass",
        TestTierKind::Quick,
        vec!["cargo check --workspace".to_string()],
        1,
        0,
        0,
        Some(1_000),
    );
    assert_eq!(passed.tier_status, TestTierStatus::TierPassed);

    let failed = build_test_tier_runner_report(
        "full-fail",
        TestTierKind::Full,
        vec!["cargo test --workspace --quiet".to_string()],
        0,
        1,
        0,
        Some(2_000),
    );
    assert_eq!(failed.tier_status, TestTierStatus::TierFailed);

    let not_run = build_test_tier_runner_report(
        "audit-skip",
        TestTierKind::Audit,
        vec!["cargo test --quiet --test rust_toolchain_cli_safety".to_string()],
        0,
        0,
        1,
        None,
    );
    assert_eq!(not_run.tier_status, TestTierStatus::TierNotRun);
    assert_eq!(not_run.skipped_count, 1);
}

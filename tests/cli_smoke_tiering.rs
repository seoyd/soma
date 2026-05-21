#[path = "support/sprint69_support.rs"]
mod support;

use soma_zero::{
    CliSmokeTieringStatus, RustToolchainModernizationRunner, build_cli_smoke_tiering_report,
};

#[test]
fn cli_smoke_tiering_retains_required_help_and_representation() {
    let report = build_cli_smoke_tiering_report();
    assert!(
        report
            .required_smoke
            .iter()
            .any(|command| command.contains("--help"))
    );
    assert!(
        report
            .representative_smoke
            .iter()
            .any(|command| command.contains("toolchain-version-report"))
    );
    assert!(
        report
            .exhaustive_smoke
            .iter()
            .any(|command| command.contains("workspace-acceptance-v2"))
    );
    assert_eq!(
        report.tiering_status,
        CliSmokeTieringStatus::SmokeTieringReady
    );
    assert_eq!(report, build_cli_smoke_tiering_report());
}

#[test]
fn cli_smoke_tiering_example_is_deterministic() {
    let config =
        support::sprint76_config_from_example("soma_cli_smoke_tiering.toml", "cli-smoke-tiering");
    let runner = RustToolchainModernizationRunner::default();
    let first = runner.run_cli_smoke_tiering(&config).expect("first report");
    let second = runner
        .run_cli_smoke_tiering(&config)
        .expect("second report");
    assert_eq!(first, second);
}

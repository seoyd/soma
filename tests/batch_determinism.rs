mod common;

use std::process::Command;

use soma_zero::BatchExperimentRunner;

#[test]
fn same_matrix_config_produces_identical_batch_report() {
    let matrix = common::batch_matrix(
        "batch-determinism",
        vec![common::dataset_entry(
            "valid",
            "generic_ohlcv_valid.csv",
            true,
        )],
        vec![common::baseline_variant("baseline_5m", true)],
    );

    let first = BatchExperimentRunner::default().run_matrix(&matrix);
    let second = BatchExperimentRunner::default().run_matrix(&matrix);

    assert_eq!(first, second);
    assert_eq!(
        first.aggregate_benchmark.to_markdown_table_string(),
        second.aggregate_benchmark.to_markdown_table_string()
    );
}

#[test]
fn batch_command_help_is_research_only_and_has_no_live_or_broker_commands() {
    let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .arg("--help")
        .output()
        .expect("cli help");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Research-only"));
    assert!(stdout.contains("batch"));
    assert!(!stdout.contains("\n  broker"));
    assert!(!stdout.contains("\n  deploy"));
    assert!(!stdout.contains("\n  websocket"));
    assert!(!stdout.contains("\n  live"));
}

mod common;

use std::process::Command;

#[test]
fn model_ops_review_closure_cli_stays_local_and_research_only() {
    let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .args([
            "model-review-close",
            "--config",
            "examples/soma_model_review_close.toml",
        ])
        .output()
        .expect("run model-review-close");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("utf8 stdout");
    assert!(stdout.contains("closure_status"));
    for forbidden in [
        "http://",
        "https://",
        "\"order\"",
        "\"account\"",
        "\"live\"",
    ] {
        assert!(
            !stdout.contains(forbidden),
            "unexpected cli token: {forbidden}"
        );
    }
}

#[test]
fn prediction_history_pack_cli_stays_local_and_research_only() {
    let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .args([
            "prediction-history-pack",
            "--config",
            "examples/soma_prediction_history_pack.toml",
        ])
        .output()
        .expect("run prediction-history-pack");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("utf8 stdout");
    assert!(stdout.contains("history_status"));
    for forbidden in [
        "http://",
        "https://",
        "\"broker\"",
        "\"train\"",
        "\"runtime\"",
    ] {
        assert!(
            !stdout.contains(forbidden),
            "unexpected cli token: {forbidden}"
        );
    }
}

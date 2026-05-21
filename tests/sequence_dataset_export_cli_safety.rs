use std::process::Command;

#[test]
fn sprint62_help_texts_include_safety_language() {
    let bin = env!("CARGO_BIN_EXE_soma_experiment");

    let export_help = Command::new(bin)
        .args(["sequence-dataset-export", "--help"])
        .output()
        .expect("run export help");
    assert!(String::from_utf8_lossy(&export_help.stdout).contains("no training"));

    let quality_help = Command::new(bin)
        .args(["sequence-dataset-quality", "--help"])
        .output()
        .expect("run quality help");
    assert!(String::from_utf8_lossy(&quality_help.stdout).contains("Research-only"));

    let drift_help = Command::new(bin)
        .args(["sequence-dataset-drift", "--help"])
        .output()
        .expect("run drift help");
    assert!(String::from_utf8_lossy(&drift_help.stdout).contains("deterministic"));

    let replay_help = Command::new(bin)
        .args(["sequence-dataset-replay-check", "--help"])
        .output()
        .expect("run replay help");
    assert!(String::from_utf8_lossy(&replay_help.stdout).contains("deterministic"));

    let bridge_help = Command::new(bin)
        .args(["external-bridge-readiness", "--help"])
        .output()
        .expect("run bridge help");
    assert!(String::from_utf8_lossy(&bridge_help.stdout).contains("import/evaluation only"));

    let gate_help = Command::new(bin)
        .args(["mamba3fin-prototype-gate", "--help"])
        .output()
        .expect("run gate help");
    assert!(String::from_utf8_lossy(&gate_help.stdout).contains("runtime stays deferred"));
}

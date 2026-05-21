use std::process::Command;

#[test]
fn sprint63_help_texts_include_safety_language() {
    let bin = env!("CARGO_BIN_EXE_soma_experiment");

    let import_help = Command::new(bin)
        .args(["external-prediction-import-v2", "--help"])
        .output()
        .expect("run import help");
    assert!(String::from_utf8_lossy(&import_help.stdout).contains("no training"));

    let evaluate_help = Command::new(bin)
        .args(["external-model-evaluate", "--help"])
        .output()
        .expect("run evaluate help");
    assert!(String::from_utf8_lossy(&evaluate_help.stdout).contains("Research-only"));

    let compare_help = Command::new(bin)
        .args(["external-vs-trinity", "--help"])
        .output()
        .expect("run compare help");
    assert!(String::from_utf8_lossy(&compare_help.stdout).contains("diagnostic comparison"));

    let ablation_help = Command::new(bin)
        .args(["external-prediction-ablation", "--help"])
        .output()
        .expect("run ablation help");
    assert!(String::from_utf8_lossy(&ablation_help.stdout).contains("diagnostic"));

    let gate_help = Command::new(bin)
        .args(["external-model-promotion-gate", "--help"])
        .output()
        .expect("run gate help");
    assert!(String::from_utf8_lossy(&gate_help.stdout).contains("never live promotion"));

    let contract_help = Command::new(bin)
        .args(["mamba3fin-contract", "--help"])
        .output()
        .expect("run contract help");
    assert!(String::from_utf8_lossy(&contract_help.stdout).contains("runtime remains deferred"));
}

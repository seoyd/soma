use std::process::Command;

#[test]
fn sprint59_help_texts_include_safety_language() {
    let bin = env!("CARGO_BIN_EXE_soma_experiment");

    let review_help = Command::new(bin)
        .args(["system-review", "--help"])
        .output()
        .expect("run system-review help");
    let review_text = String::from_utf8_lossy(&review_help.stdout);
    assert!(review_text.contains("paper-only"));
    assert!(review_text.contains("live trading"));

    let diff_help = Command::new(bin)
        .args(["system-benchmark-diff", "--help"])
        .output()
        .expect("run system-benchmark-diff help");
    let diff_text = String::from_utf8_lossy(&diff_help.stdout);
    assert!(diff_text.contains("deterministic artifact diff"));

    let checklist_help = Command::new(bin)
        .args(["manual-ship-checklist", "--help"])
        .output()
        .expect("run manual-ship-checklist help");
    let checklist_text = String::from_utf8_lossy(&checklist_help.stdout);
    assert!(checklist_text.contains("manual paper-ops gate"));

    let gate_help = Command::new(bin)
        .args(["system-ship-gate", "--help"])
        .output()
        .expect("run system-ship-gate help");
    let gate_text = String::from_utf8_lossy(&gate_help.stdout);
    assert!(gate_text.contains("paper-ops-monitoring only"));
}

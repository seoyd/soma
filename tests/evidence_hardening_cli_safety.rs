use std::process::Command;

#[test]
fn sprint60_help_texts_include_safety_language() {
    let bin = env!("CARGO_BIN_EXE_soma_experiment");

    let hardening_help = Command::new(bin)
        .args(["evidence-hardening", "--help"])
        .output()
        .expect("run evidence-hardening help");
    let hardening_text = String::from_utf8_lossy(&hardening_help.stdout);
    assert!(hardening_text.contains("paper-only"));

    let outcome_help = Command::new(bin)
        .args(["outcome-link-coverage", "--help"])
        .output()
        .expect("run outcome-link-coverage help");
    let outcome_text = String::from_utf8_lossy(&outcome_help.stdout);
    assert!(outcome_text.contains("live trading"));

    let counterfactual_help = Command::new(bin)
        .args(["counterfactual-coverage", "--help"])
        .output()
        .expect("run counterfactual-coverage help");
    let counterfactual_text = String::from_utf8_lossy(&counterfactual_help.stdout);
    assert!(counterfactual_text.contains("paper-only"));

    let review_help = Command::new(bin)
        .args(["review-ergonomics", "--help"])
        .output()
        .expect("run review-ergonomics help");
    let review_text = String::from_utf8_lossy(&review_help.stdout);
    assert!(review_text.contains("owner review"));

    let ui_help = Command::new(bin)
        .args(["ui-framework-decision", "--help"])
        .output()
        .expect("run ui-framework-decision help");
    let ui_text = String::from_utf8_lossy(&ui_help.stdout);
    assert!(ui_text.contains("local UI"));

    let mamba_help = Command::new(bin)
        .args(["mamba-application-timing", "--help"])
        .output()
        .expect("run mamba-application-timing help");
    let mamba_text = String::from_utf8_lossy(&mamba_help.stdout);
    assert!(mamba_text.contains("runtime remains deferred"));
}

use std::process::Command;

#[test]
fn sprint61_help_texts_include_safety_language() {
    let bin = env!("CARGO_BIN_EXE_soma_experiment");

    let plan_help = Command::new(bin)
        .args(["kis-evidence-expansion-plan-v2", "--help"])
        .output()
        .expect("run plan help");
    assert!(
        String::from_utf8_lossy(&plan_help.stdout).contains("operator live data stays disabled")
    );

    let closure_help = Command::new(bin)
        .args(["kis-evidence-closure", "--help"])
        .output()
        .expect("run closure help");
    assert!(
        String::from_utf8_lossy(&closure_help.stdout).contains("never a live-trading approval")
    );

    let outcome_help = Command::new(bin)
        .args(["outcome-link-depth-close-v2", "--help"])
        .output()
        .expect("run outcome help");
    assert!(String::from_utf8_lossy(&outcome_help.stdout).contains("no-lookahead"));

    let owner_help = Command::new(bin)
        .args(["owner-review-discipline-v2", "--help"])
        .output()
        .expect("run owner help");
    assert!(String::from_utf8_lossy(&owner_help.stdout).contains("manual-only"));

    let sequence_help = Command::new(bin)
        .args(["sequence-readiness-hardening", "--help"])
        .output()
        .expect("run sequence help");
    assert!(String::from_utf8_lossy(&sequence_help.stdout).contains("no training"));

    let preview_help = Command::new(bin)
        .args(["sequence-window-preview", "--help"])
        .output()
        .expect("run preview help");
    assert!(String::from_utf8_lossy(&preview_help.stdout).contains("preview"));

    let proof_help = Command::new(bin)
        .args(["no-lookahead-sequence-proof", "--help"])
        .output()
        .expect("run proof help");
    assert!(String::from_utf8_lossy(&proof_help.stdout).contains("no-lookahead"));
}

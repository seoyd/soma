#[path = "support/sprint69_support.rs"]
mod support;

#[test]
fn sprint74_outputs_are_deterministic() {
    let first = support::run_sprint74_bundle(
        "soma_real_evidence_followup.toml",
        "real-evidence-followup-determinism-a",
    );
    let second = support::run_sprint74_bundle(
        "soma_real_evidence_followup.toml",
        "real-evidence-followup-determinism-b",
    );
    assert_eq!(first, second);
}

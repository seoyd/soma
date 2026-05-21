mod support;

use support::sprint105_support::run_sprint105;

#[test]
fn final_verification_gate_v2_is_ready_with_explicit_warnings() {
    let bundle = run_sprint105(
        "soma_final_verification_gate_v2.toml",
        "final_verification_gate_v2",
    );
    assert!(
        bundle
            .final_verification_gate_v2
            .gate_status
            .contains("Ready")
    );
    assert!(!bundle.final_verification_gate_v2.full_workspace_accepted);
}

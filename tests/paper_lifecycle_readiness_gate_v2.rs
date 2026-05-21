mod support;

use support::sprint105_support::run_sprint105;

#[test]
fn paper_lifecycle_readiness_gate_v2_stays_paper_only() {
    let bundle = run_sprint105(
        "soma_paper_lifecycle_readiness_gate_v2.toml",
        "paper_lifecycle_readiness_gate_v2",
    );
    let gate = &bundle.paper_lifecycle_readiness_gate_v2;
    assert!(!gate.live_lifecycle_allowed);
    assert!(
        gate.gate_status == "PaperLifecycleReadyWithWarnings"
            || gate.gate_status == "PaperLifecycleReady"
            || gate.gate_status == "PaperLifecycleBlocked"
    );
}

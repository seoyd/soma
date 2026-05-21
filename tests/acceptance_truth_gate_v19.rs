mod support;

use soma_zero::{
    AcceptanceTruthGateV19, CommandObservation, build_acceptance_truth_gate_v19,
    build_truthful_full_workspace_attempt_v19, build_workspace_full_acceptance_gate_v19,
};
use support::sprint118_support::{read_fixture, run_sprint118};

#[test]
fn acceptance_truth_gate_v19_requires_full_finished_and_passed() {
    let bundle = run_sprint118(
        "soma_acceptance_truth_gate_v19.toml",
        "acceptance-truth-gate-v19",
    );
    let expected: AcceptanceTruthGateV19 =
        read_fixture("sprint118_data/acceptance_truth_gate_v19_expected.json");
    assert_eq!(bundle.acceptance_truth_gate_v19, expected);
    let full = build_truthful_full_workspace_attempt_v19(
        Some(&CommandObservation {
            attempted: true,
            finished: true,
            passed: Some(true),
            duration_ms: Some(1),
            timeout_ms: Some(420000),
            exit_code: Some(0),
            timed_out: false,
            stdout: String::new(),
        }),
        Some(420000),
    );
    let gate = build_workspace_full_acceptance_gate_v19(&full);
    assert!(build_acceptance_truth_gate_v19(&gate).can_claim_full_acceptance);
}

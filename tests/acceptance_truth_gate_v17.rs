mod support;

use soma_zero::{
    AcceptanceTruthGateV17, CommandObservation, Sprint115SummaryFixture,
    build_acceptance_truth_gate_v17, build_real_full_workspace_observation_attempt_v17,
    build_real_no_run_observation_attempt_v17, build_workspace_full_acceptance_gate_v17,
    build_workspace_no_run_recovery_gate_v17,
};
use support::sprint116_support::{read_fixture, run_sprint116};

#[test]
fn acceptance_truth_gate_v17_matches_expected() {
    let bundle = run_sprint116(
        "soma_acceptance_truth_gate_v17.toml",
        "acceptance-truth-gate-v17",
    );
    let expected: AcceptanceTruthGateV17 =
        read_fixture("sprint116_data/acceptance_truth_gate_v17_expected.json");
    assert_eq!(bundle.acceptance_truth_gate_v17, expected);
    assert!(!bundle.acceptance_truth_gate_v17.can_claim_full_acceptance);
}

#[test]
fn acceptance_truth_requires_full_finished_and_passed() {
    let summary = Sprint115SummaryFixture::default();
    let no_run = build_real_no_run_observation_attempt_v17(None, Some(420_000));
    let no_run_gate = build_workspace_no_run_recovery_gate_v17(&no_run, &summary);
    assert!(!no_run_gate.timed_out);
    let full = build_real_full_workspace_observation_attempt_v17(
        Some(&CommandObservation {
            attempted: true,
            finished: true,
            passed: Some(true),
            duration_ms: Some(1_000),
            timeout_ms: Some(420_000),
            exit_code: Some(0),
            timed_out: false,
            stdout: String::new(),
        }),
        Some(420_000),
    );
    let full_gate = build_workspace_full_acceptance_gate_v17(&full, &summary);
    let acceptance = build_acceptance_truth_gate_v17(&summary, &no_run_gate, &full_gate);
    assert!(acceptance.can_claim_full_acceptance);
}

mod support;

use soma_zero::{
    AcceptanceTruthGateV16, Sprint114SummaryFixture, build_workspace_no_run_recovery_gate_v16,
};
use support::sprint115_support::{read_fixture, run_sprint115};

#[test]
fn acceptance_truth_gate_v16_matches_expected() {
    let bundle = run_sprint115(
        "soma_acceptance_truth_gate_v16.toml",
        "acceptance-truth-gate-v16",
    );
    let expected: AcceptanceTruthGateV16 =
        read_fixture("sprint115_data/acceptance_truth_gate_v16_expected.json");
    assert_eq!(bundle.acceptance_truth_gate_v16, expected);
    assert!(!bundle.acceptance_truth_gate_v16.can_claim_full_acceptance);
}

#[test]
fn no_run_gate_distinguishes_not_run_from_timeout() {
    let not_run_summary = Sprint114SummaryFixture {
        no_run_status: "NoRunNotRun".to_string(),
        no_run_exit_code: None,
        ..Sprint114SummaryFixture::default()
    };

    let gate = build_workspace_no_run_recovery_gate_v16(&not_run_summary);

    assert_eq!(gate.gate_status, "NoRunStillBlocked");
    assert!(!gate.finished);
    assert!(!gate.passed);
    assert!(!gate.recovered);
    assert!(!gate.timed_out);

    let timed_out_gate =
        build_workspace_no_run_recovery_gate_v16(&Sprint114SummaryFixture::default());
    assert!(timed_out_gate.timed_out);
}

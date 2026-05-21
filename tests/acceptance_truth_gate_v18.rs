mod support;

use soma_zero::{
    AcceptanceTruthGateV18, CommandObservation, Sprint116SummaryFixture,
    build_acceptance_truth_gate_v18, build_real_full_workspace_execution_report_v18,
    build_real_no_run_execution_report_v18, build_workspace_full_acceptance_gate_v18,
    build_workspace_no_run_recovery_gate_v18,
};
use support::sprint117_support::{read_fixture, run_sprint117};

#[test]
fn acceptance_truth_gate_v18_requires_full_finished_and_passed() {
    let bundle = run_sprint117(
        "soma_acceptance_truth_gate_v18.toml",
        "acceptance-truth-gate-v18",
    );
    let expected: AcceptanceTruthGateV18 =
        read_fixture("sprint117_data/acceptance_truth_gate_v18_expected.json");
    assert_eq!(bundle.acceptance_truth_gate_v18, expected);
    let summary = Sprint116SummaryFixture::default();
    let no_run = build_real_no_run_execution_report_v18(None, Some(420000), None);
    let no_run_gate = build_workspace_no_run_recovery_gate_v18(&no_run);
    let full = build_real_full_workspace_execution_report_v18(
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
        Some((0, 0)),
    );
    let full_gate = build_workspace_full_acceptance_gate_v18(&summary, &full);
    let acceptance = build_acceptance_truth_gate_v18(&summary, &no_run_gate, &full_gate);
    assert!(acceptance.can_claim_full_acceptance);
}

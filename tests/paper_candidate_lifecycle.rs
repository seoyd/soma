mod support;

use serde_json::to_value;
use soma_zero::PaperCandidateLifecycleState;
use support::sprint104_support::{read_fixture, run_sprint104};

#[test]
fn paper_candidate_lifecycle_state_machine_contains_all_required_states() {
    let bundle = run_sprint104(
        "soma_sprint104_dual_agent_paper_lifecycle.toml",
        "paper_candidate_lifecycle",
    );
    let actual = to_value(&bundle.paper_candidate_lifecycle_state_machine).expect("actual");
    let expected: serde_json::Value =
        read_fixture("sprint104_data/paper_candidate_lifecycle_expected.json");
    assert_eq!(actual, expected);
    for state in [
        PaperCandidateLifecycleState::Watch,
        PaperCandidateLifecycleState::Candidate,
        PaperCandidateLifecycleState::DebateOpen,
        PaperCandidateLifecycleState::NeedMoreEvidence,
        PaperCandidateLifecycleState::NoTrade,
        PaperCandidateLifecycleState::RiskDenied,
        PaperCandidateLifecycleState::PaperApproved,
        PaperCandidateLifecycleState::PaperRejected,
        PaperCandidateLifecycleState::Cooldown,
        PaperCandidateLifecycleState::ArchivedPaperOnly,
    ] {
        let label = format!("{state:?}");
        assert!(
            bundle
                .control_tower_paper_candidate_lifecycle_panel
                .candidate_state_summary
                .contains_key(&label)
        );
    }
    assert!(
        bundle
            .paper_candidate_lifecycle_state_machine
            .forbidden_transitions
            .iter()
            .any(|transition| transition.contains("LiveExecution"))
    );
    assert!(
        !bundle
            .paper_candidate_lifecycle_state_machine
            .live_execution_allowed
    );
}

#[test]
fn paper_candidate_lifecycle_transitions_require_risk_governor() {
    let bundle = run_sprint104(
        "soma_sprint104_dual_agent_paper_lifecycle.toml",
        "paper_candidate_lifecycle_risk",
    );
    assert!(
        bundle
            .paper_candidate_lifecycle_state_machine
            .risk_governor_required_transitions
            .iter()
            .any(|transition| transition == "DebateOpen->PaperApproved")
    );
    assert!(
        bundle
            .paper_candidate_lifecycle_state_machine
            .risk_governor_required_transitions
            .iter()
            .any(|transition| transition == "DebateOpen->PaperRejected")
    );
}

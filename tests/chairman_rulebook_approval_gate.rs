mod support;

use std::fs;
use std::path::PathBuf;

use soma_zero::{ChairmanRulebookApprovalGate, ChairmanRulebookApprovalStatus};
use support::sprint100_support::run_sprint100;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("sprint100_data")
        .join(name)
}

#[test]
fn chairman_rulebook_approval_gate_matches_expected_fixture() {
    let bundle = run_sprint100(
        "soma_chairman_rulebook_approval_gate.toml",
        "chairman-rulebook-approval-gate",
    );
    let expected: ChairmanRulebookApprovalGate = serde_json::from_str(
        &fs::read_to_string(fixture_path("rulebook_approval_gate_expected.json"))
            .expect("read fixture"),
    )
    .expect("parse fixture");
    assert_eq!(bundle.chairman_rulebook_approval_gate, expected);
    assert_eq!(
        bundle.chairman_rulebook_approval_gate.approval_status,
        ChairmanRulebookApprovalStatus::RulebookApprovedForPaper
    );
    assert!(
        bundle
            .chairman_rulebook_approval_gate
            .can_activate_for_paper
    );
    assert!(!bundle.chairman_rulebook_approval_gate.can_activate_for_live);
    assert!(bundle.chairman_rulebook_v2_draft.live_use_forbidden);
}

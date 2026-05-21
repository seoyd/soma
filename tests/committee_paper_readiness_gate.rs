mod support;

use std::fs;
use std::path::PathBuf;

use soma_zero::CommitteePaperReadinessGate;
use support::sprint100_support::run_sprint100;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("sprint100_data")
        .join(name)
}

#[test]
fn committee_paper_readiness_gate_matches_expected_fixture() {
    let bundle = run_sprint100(
        "soma_committee_paper_readiness_gate.toml",
        "committee-paper-readiness-gate",
    );
    let expected: CommitteePaperReadinessGate = serde_json::from_str(
        &fs::read_to_string(fixture_path("committee_paper_readiness_gate_expected.json"))
            .expect("read fixture"),
    )
    .expect("parse fixture");
    assert_eq!(bundle.committee_paper_readiness_gate, expected);
    assert!(!bundle.committee_paper_readiness_gate.live_loop_allowed);
}

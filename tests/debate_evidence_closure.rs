mod support;

use std::fs;
use std::path::PathBuf;

use soma_zero::DebateNeedsMoreEvidenceClosureReport;
use support::sprint100_support::run_sprint100;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("sprint100_data")
        .join(name)
}

#[test]
fn debate_evidence_closure_matches_expected_fixture() {
    let bundle = run_sprint100(
        "soma_debate_evidence_closure.toml",
        "debate-evidence-closure",
    );
    let expected: DebateNeedsMoreEvidenceClosureReport = serde_json::from_str(
        &fs::read_to_string(fixture_path("debate_evidence_closure_expected.json"))
            .expect("read fixture"),
    )
    .expect("parse fixture");
    assert_eq!(bundle.debate_needs_more_evidence_closure_report, expected);
}

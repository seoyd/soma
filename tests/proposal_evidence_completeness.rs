mod support;

use std::fs;
use std::path::PathBuf;

use soma_zero::ProposalEvidenceCompletenessReport;
use support::sprint100_support::run_sprint100;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("sprint100_data")
        .join(name)
}

#[test]
fn proposal_evidence_completeness_matches_expected_fixture() {
    let bundle = run_sprint100(
        "soma_proposal_evidence_completeness.toml",
        "proposal-evidence-completeness",
    );
    let expected: ProposalEvidenceCompletenessReport = serde_json::from_str(
        &fs::read_to_string(fixture_path("proposal_evidence_completeness_expected.json"))
            .expect("read fixture"),
    )
    .expect("parse fixture");
    assert_eq!(bundle.proposal_evidence_completeness_report, expected);
}

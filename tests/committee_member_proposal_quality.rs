mod support;

use std::fs;
use std::path::PathBuf;

use soma_zero::{CommitteeMemberProposalQualityReport, CommitteeMemberProposalQualityStatus};
use support::sprint99_support::run_sprint99;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("sprint99_data")
        .join(name)
}

#[test]
fn committee_member_proposal_quality_matches_expected_fixture() {
    let bundle = run_sprint99(
        "soma_committee_member_proposal_quality.toml",
        "committee-member-proposal-quality",
    );
    let expected: CommitteeMemberProposalQualityReport = serde_json::from_str(
        &fs::read_to_string(fixture_path("member_proposal_quality_expected.json"))
            .expect("read fixture"),
    )
    .expect("parse fixture");
    assert_eq!(bundle.committee_member_proposal_quality_report, expected);
    assert_eq!(
        bundle
            .committee_member_proposal_quality_report
            .quality_status,
        CommitteeMemberProposalQualityStatus::ProposalQualityReadyWithWarnings
    );
    assert!(
        bundle
            .committee_member_proposal_quality_report
            .confidence_bounds_valid
    );
}

mod support;

use std::fs;
use std::path::PathBuf;

use soma_zero::{
    CommitteeConsensusState, CommitteeDebateQualityReport, CommitteeDebateQualityStatus,
};
use support::sprint99_support::run_sprint99;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("sprint99_data")
        .join(name)
}

#[test]
fn committee_debate_quality_matches_expected_fixture() {
    let bundle = run_sprint99(
        "soma_committee_debate_quality.toml",
        "committee-debate-quality",
    );
    let expected: CommitteeDebateQualityReport = serde_json::from_str(
        &fs::read_to_string(fixture_path("debate_quality_expected.json")).expect("read fixture"),
    )
    .expect("parse fixture");
    assert_eq!(bundle.committee_debate_quality_report, expected);
    assert_eq!(
        bundle.committee_debate_quality_report.debate_quality_status,
        CommitteeDebateQualityStatus::DebateNeedsMoreEvidence
    );
    assert_eq!(
        bundle.committee_debate_quality_report.consensus_state,
        CommitteeConsensusState::NeedMoreEvidence
    );
    assert!(bundle.committee_debate_quality_report.disagreement_present);
}

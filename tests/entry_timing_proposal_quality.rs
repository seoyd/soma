mod support;

use std::fs;
use std::path::PathBuf;

use soma_zero::{EntryTimingProposalQualityReport, EntryTimingProposalQualityStatus};
use support::sprint99_support::run_sprint99;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("sprint99_data")
        .join(name)
}

#[test]
fn entry_timing_proposal_quality_matches_expected_fixture() {
    let bundle = run_sprint99(
        "soma_entry_timing_proposal_quality.toml",
        "entry-timing-proposal-quality",
    );
    let expected: EntryTimingProposalQualityReport = serde_json::from_str(
        &fs::read_to_string(fixture_path("entry_timing_quality_expected.json"))
            .expect("read fixture"),
    )
    .expect("parse fixture");
    assert_eq!(bundle.entry_timing_proposal_quality_report, expected);
    assert_eq!(
        bundle
            .entry_timing_proposal_quality_report
            .timing_quality_status,
        EntryTimingProposalQualityStatus::EntryTimingQualityReady
    );
    assert!(
        bundle
            .entry_timing_proposal_quality_report
            .paper_only_timing_confirmed
    );
}

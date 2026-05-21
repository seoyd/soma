mod support;

use std::fs;
use std::path::PathBuf;

use soma_zero::{ProposalQualityWarningClosureReport, ProposalQualityWarningClosureStatus};
use support::sprint100_support::run_sprint100;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("sprint100_data")
        .join(name)
}

#[test]
fn proposal_warning_closure_matches_expected_fixture() {
    let bundle = run_sprint100(
        "soma_proposal_warning_closure.toml",
        "proposal-warning-closure",
    );
    let expected: ProposalQualityWarningClosureReport = serde_json::from_str(
        &fs::read_to_string(fixture_path("proposal_warning_closure_expected.json"))
            .expect("read fixture"),
    )
    .expect("parse fixture");
    assert_eq!(bundle.proposal_quality_warning_closure_report, expected);
    assert_eq!(
        bundle
            .proposal_quality_warning_closure_report
            .closure_status,
        ProposalQualityWarningClosureStatus::ProposalWarningsClosedWithMinorNotes
    );
    assert_eq!(
        bundle
            .proposal_quality_warning_closure_report
            .closed_warning_count,
        1
    );
    assert_eq!(
        bundle
            .proposal_quality_warning_closure_report
            .remaining_warning_count,
        0
    );
}

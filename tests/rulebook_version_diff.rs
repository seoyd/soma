mod support;

use std::fs;
use std::path::PathBuf;

use soma_zero::{RulebookVersionDiffReport, RulebookVersionDiffStatus};
use support::sprint99_support::run_sprint99;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("sprint99_data")
        .join(name)
}

#[test]
fn rulebook_version_diff_matches_expected_fixture() {
    let bundle = run_sprint99("soma_rulebook_version_diff.toml", "rulebook-version-diff");
    let expected: RulebookVersionDiffReport = serde_json::from_str(
        &fs::read_to_string(fixture_path("rulebook_diff_expected.json")).expect("read fixture"),
    )
    .expect("parse fixture");
    assert_eq!(bundle.rulebook_version_diff_report, expected);
    assert_eq!(
        bundle.rulebook_version_diff_report.diff_status,
        RulebookVersionDiffStatus::RulebookDiffReadyWithWarnings
    );
    assert!(bundle.rulebook_version_diff_report.changed_rules >= 1);
}

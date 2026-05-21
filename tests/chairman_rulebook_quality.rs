mod support;

use std::fs;
use std::path::PathBuf;

use soma_zero::{ChairmanRulebookQualityReport, ChairmanRulebookQualityStatus};
use support::sprint99_support::run_sprint99;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("sprint99_data")
        .join(name)
}

#[test]
fn chairman_rulebook_quality_matches_expected_fixture() {
    let bundle = run_sprint99(
        "soma_chairman_rulebook_quality.toml",
        "chairman-rulebook-quality",
    );
    let expected: ChairmanRulebookQualityReport = serde_json::from_str(
        &fs::read_to_string(fixture_path("rulebook_quality_expected.json")).expect("read fixture"),
    )
    .expect("parse fixture");
    assert_eq!(bundle.chairman_rulebook_quality_report, expected);
    assert_eq!(
        bundle
            .chairman_rulebook_quality_report
            .rulebook_quality_status,
        ChairmanRulebookQualityStatus::UnsafeRuleDetected
    );
    assert!(
        bundle
            .chairman_rulebook_quality_report
            .live_use_forbidden_confirmed
    );
}

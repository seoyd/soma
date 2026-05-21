mod support;

use std::fs;
use std::path::PathBuf;

use soma_zero::DebateDissentCoverageReport;
use support::sprint100_support::run_sprint100;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("sprint100_data")
        .join(name)
}

#[test]
fn debate_dissent_coverage_matches_expected_fixture() {
    let bundle = run_sprint100(
        "soma_debate_dissent_coverage.toml",
        "debate-dissent-coverage",
    );
    let expected: DebateDissentCoverageReport = serde_json::from_str(
        &fs::read_to_string(fixture_path("debate_dissent_expected.json")).expect("read fixture"),
    )
    .expect("parse fixture");
    assert_eq!(bundle.debate_dissent_coverage_report, expected);
}

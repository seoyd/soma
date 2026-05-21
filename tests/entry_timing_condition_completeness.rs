mod support;

use std::fs;
use std::path::PathBuf;

use soma_zero::EntryTimingConditionCompletenessReport;
use support::sprint100_support::run_sprint100;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("sprint100_data")
        .join(name)
}

#[test]
fn entry_timing_condition_completeness_matches_expected_fixture() {
    let bundle = run_sprint100(
        "soma_entry_timing_condition_completeness.toml",
        "entry-timing-condition-completeness",
    );
    let expected: EntryTimingConditionCompletenessReport = serde_json::from_str(
        &fs::read_to_string(fixture_path("entry_timing_conditions_expected.json"))
            .expect("read fixture"),
    )
    .expect("parse fixture");
    assert_eq!(bundle.entry_timing_condition_completeness_report, expected);
}

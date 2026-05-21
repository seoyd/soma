mod support;

use std::fs;
use std::path::PathBuf;

use soma_zero::ChairmanUnsafeRuleClosureReport;
use support::sprint100_support::run_sprint100;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("sprint100_data")
        .join(name)
}

#[test]
fn chairman_unsafe_rule_closure_matches_expected_fixture() {
    let bundle = run_sprint100(
        "soma_chairman_unsafe_rule_closure.toml",
        "chairman-unsafe-rule-closure",
    );
    let expected: ChairmanUnsafeRuleClosureReport = serde_json::from_str(
        &fs::read_to_string(fixture_path("chairman_unsafe_rule_closure_expected.json"))
            .expect("read fixture"),
    )
    .expect("parse fixture");
    assert_eq!(bundle.chairman_unsafe_rule_closure_report, expected);
}

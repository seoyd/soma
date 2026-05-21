mod support;

use serde_json::Value;
use std::fs;
use support::sprint100_support::run_sprint100;
use support::sprint101_support::run_sprint101;

#[test]
fn investor_unverified_claim_filter_matches_fixture() {
    let _ = run_sprint100(
        "soma_sprint100_committee_closure.toml",
        "investor_unverified_claim_filter-sprint100-base",
    );
    let bundle = run_sprint101(
        "soma_sprint101_investor_archetype_ingest.toml",
        "investor_unverified_claim_filter",
    );
    let expected: Value = serde_json::from_str(
        &fs::read_to_string("examples/sprint101_data/unverified_claim_filter_expected.json")
            .expect("read fixture"),
    )
    .expect("parse fixture");
    assert_eq!(
        serde_json::to_value(&bundle.investor_unverified_claim_filter_report).expect("value"),
        expected
    );
}

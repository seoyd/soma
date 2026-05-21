mod support;

use support::sprint108_support::{read_fixture, run_sprint108};

#[test]
fn equivalent_coverage_proof_matches_expected_fixture() {
    let bundle = run_sprint108(
        "soma_equivalent_coverage_proof_v1.toml",
        "equivalent-coverage-proof-v1",
    );
    let actual = serde_json::to_value(&bundle.equivalent_coverage_proof_report_v1).expect("actual");
    let expected: serde_json::Value =
        read_fixture("sprint108_data/equivalent_coverage_proof_expected.json");
    assert_eq!(actual, expected);
    assert_eq!(
        bundle.equivalent_coverage_proof_report_v1.proof_status,
        "EquivalentCoverageProven"
    );
    assert_eq!(
        bundle
            .equivalent_coverage_proof_report_v1
            .coverage_gap_count,
        0
    );
    assert_eq!(
        bundle
            .equivalent_coverage_proof_report_v1
            .destination_targets,
        vec!["tests/shared_fixture_harness_application_v1.rs".to_string()]
    );
}

#[test]
fn coverage_gap_blocks_retirement() {
    let bundle = run_sprint108(
        "soma_equivalent_coverage_proof_v1.toml",
        "equivalent-coverage-proof-v1-gap",
    );
    let mut report = bundle.equivalent_coverage_proof_report_v1;
    report.coverage_gap_count = 1;
    assert!(report.coverage_gap_count > 0);
}

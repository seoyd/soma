mod support;

use support::sprint110_support::run_sprint110;

#[test]
fn equivalent_coverage_proof_v3_carries_validation_refs() {
    let bundle = run_sprint110(
        "soma_equivalent_coverage_proof_v3.toml",
        "equivalent-coverage-proof-v3",
    );
    let report = bundle.equivalent_coverage_proof_report_v3;
    assert_eq!(report.proof_status, "EquivalentCoverageProven");
    assert_eq!(report.coverage_gap_count, 0);
    assert_eq!(report.cumulative_coverage_gap_count, 0);
    assert_eq!(
        report.retired_targets,
        vec!["tests/shared_toml_builder_application_v1.rs".to_string()]
    );
    assert_eq!(
        report.destination_targets,
        vec!["tests/shared_fixture_harness_application_v1.rs".to_string()]
    );
    assert_eq!(
        report.sprint109_validation_refs,
        vec!["sprint109-validation-reconciliation".to_string()]
    );
}

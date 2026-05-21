mod support;

use soma_zero::Sprint117BaselineTruthImportReport;
use support::sprint118_support::{read_fixture, run_sprint118};

#[test]
fn sprint117_baseline_truth_import_matches_expected() {
    let bundle = run_sprint118(
        "soma_sprint117_baseline_truth_import.toml",
        "sprint117-baseline-truth-import",
    );
    let expected: Sprint117BaselineTruthImportReport =
        read_fixture("sprint118_data/baseline_truth_import_expected.json");
    assert_eq!(bundle.sprint117_baseline_truth_import_report, expected);
    assert!(
        bundle
            .sprint117_baseline_truth_import_report
            .focused_tests_passed
    );
    assert!(
        bundle
            .sprint117_baseline_truth_import_report
            .cli_smoke_passed
    );
    assert!(
        bundle
            .sprint117_baseline_truth_import_report
            .cargo_check_passed
    );
    assert!(
        bundle
            .sprint117_baseline_truth_import_report
            .cargo_build_passed
    );
    assert!(
        !bundle
            .sprint117_baseline_truth_import_report
            .imported_as_full_acceptance
    );
}

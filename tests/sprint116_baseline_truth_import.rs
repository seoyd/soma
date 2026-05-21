mod support;

use soma_zero::Sprint116BaselineTruthImportReport;
use support::sprint117_support::{read_fixture, run_sprint117};

#[test]
fn sprint116_baseline_truth_import_matches_expected() {
    let bundle = run_sprint117(
        "soma_sprint116_baseline_truth_import.toml",
        "sprint116-baseline-truth-import",
    );
    let expected: Sprint116BaselineTruthImportReport =
        read_fixture("sprint117_data/baseline_truth_import_expected.json");
    assert_eq!(bundle.sprint116_baseline_truth_import_report, expected);
    assert!(
        !bundle
            .sprint116_baseline_truth_import_report
            .imported_as_full_acceptance
    );
}

mod support;

use soma_zero::Sprint110BaselineTruthImportReport;
use support::sprint111_support::{read_fixture, run_sprint111};

#[test]
fn sprint110_truth_import_is_supporting_only_and_deterministic() {
    let bundle = run_sprint111(
        "soma_sprint110_baseline_truth_import.toml",
        "sprint110-truth-import",
    );
    let expected: Sprint110BaselineTruthImportReport =
        read_fixture("sprint111_data/sprint110_truth_import_expected.json");
    assert_eq!(bundle.sprint110_baseline_truth_import_report, expected);
    assert!(
        !bundle
            .sprint110_baseline_truth_import_report
            .imported_as_full_acceptance
    );
    assert!(
        bundle
            .sprint110_baseline_truth_import_report
            .no_run_timed_out
    );
    assert!(
        bundle
            .sprint110_baseline_truth_import_report
            .full_workspace_timed_out
    );
}

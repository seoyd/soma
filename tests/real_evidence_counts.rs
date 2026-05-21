mod common;

use soma_zero::RealEvidenceClosureRunner;

#[test]
fn real_only_counts_exclude_synthetic_and_test_sources() {
    common::ensure_sprint15_report();
    let report = RealEvidenceClosureRunner::default().run(&common::real_evidence_config(
        "real-evidence-counts",
        vec![
            common::real_local_test_entry("real_alt", "generic_ohlcv_valid_alt.csv"),
            common::synthetic_entry("synthetic_alt", "generic_ohlcv_valid_alt.csv"),
        ],
    ));
    assert!(report.source_evidence_summary.real_local_outcome_records > 0);
    assert!(report.source_evidence_summary.synthetic_outcome_records > 0);
    assert_eq!(
        report.source_evidence_summary.readiness_eligible_datasets,
        1
    );
    assert_eq!(report.source_evidence_summary.real_local_datasets, 1);
    assert!(report.source_evidence_summary.synthetic_fixture_datasets >= 1);
}

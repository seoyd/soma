mod common;

use soma_zero::{RealEvidenceClosureRunner, RealEvidenceRecommendation};

#[test]
fn synthetic_only_input_does_not_count_for_real_readiness() {
    common::ensure_sprint15_report();
    let report = RealEvidenceClosureRunner::default().run(&common::real_evidence_config(
        "synthetic-only",
        vec![common::synthetic_entry(
            "synthetic_alt",
            "generic_ohlcv_valid_alt.csv",
        )],
    ));
    assert_eq!(
        report.final_recommendation,
        RealEvidenceRecommendation::MissingRealLocalData
    );
    assert_eq!(report.real_only_evidence_status.dataset_count, 0);
    assert_eq!(report.real_only_evidence_status.outcome_count, 0);
}

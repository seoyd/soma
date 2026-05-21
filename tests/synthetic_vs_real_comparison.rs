mod common;

use soma_zero::{RealEvidenceClosureRunner, real_evidence_report_to_text};

#[test]
fn comparison_is_not_conclusive_when_real_data_is_missing() {
    common::ensure_sprint15_report();
    let report = RealEvidenceClosureRunner::default().run(&common::real_evidence_config(
        "comparison-missing-real",
        vec![common::synthetic_entry(
            "synthetic_alt",
            "generic_ohlcv_valid_alt.csv",
        )],
    ));
    let comparison = report
        .synthetic_vs_real_comparison
        .expect("comparison report");
    assert!(!comparison.comparable);
}

#[test]
fn comparison_rendering_is_deterministic() {
    common::ensure_sprint15_report();
    let config = common::real_evidence_config(
        "comparison-deterministic",
        vec![common::real_local_test_entry(
            "real_alt",
            "generic_ohlcv_valid_alt.csv",
        )],
    );
    let report_a = RealEvidenceClosureRunner::default().run(&config);
    let report_b = RealEvidenceClosureRunner::default().run(&config);
    assert_eq!(
        report_a.synthetic_vs_real_comparison,
        report_b.synthetic_vs_real_comparison
    );
    assert_eq!(
        real_evidence_report_to_text(&report_a),
        real_evidence_report_to_text(&report_b)
    );
}

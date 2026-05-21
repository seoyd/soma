mod common;

use soma_zero::{RealEvidenceClosureRunner, RealEvidenceRecommendation};

#[test]
fn no_real_local_data_yields_missing_real_local_data() {
    common::ensure_sprint15_report();
    let report = RealEvidenceClosureRunner::default()
        .run(&common::real_evidence_config("no-real-data", Vec::new()));
    assert_eq!(
        report.final_recommendation,
        RealEvidenceRecommendation::MissingRealLocalData
    );
}

#[test]
fn bad_real_local_data_recommends_improve_data_first() {
    let report = RealEvidenceClosureRunner::default().run(&common::real_evidence_config(
        "bad-real-data",
        vec![common::real_local_test_entry(
            "bad_real_gap",
            "generic_ohlcv_bad_ohlc.csv",
        )],
    ));
    assert_eq!(
        report.final_recommendation,
        RealEvidenceRecommendation::ImproveDataFirst
    );
}

#[test]
fn insufficient_real_variant_count_stays_conservative() {
    let mut config = common::real_evidence_config(
        "insufficient-real-variants",
        vec![common::real_local_test_entry(
            "real_alt",
            "generic_ohlcv_valid_alt.csv",
        )],
    );
    config.min_real_local_comparable_variants = 3;
    let report = RealEvidenceClosureRunner::default().run(&config);
    assert_eq!(
        report.final_recommendation,
        RealEvidenceRecommendation::NeedMoreExperiments
    );
}

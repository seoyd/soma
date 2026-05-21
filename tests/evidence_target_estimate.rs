mod common;

use soma_zero::{CandleCsvLoader, ReasonCode, estimate_evidence_targets};

#[test]
fn enough_rows_produce_positive_outcome_estimate() {
    let config = common::onboarding_config("estimate-positive", "generic_ohlcv_valid_alt.csv");
    let loader = CandleCsvLoader::default();
    let loaded = loader
        .load_from_path(
            &common::fixture_path("generic_ohlcv_valid_alt.csv"),
            &config.build_csv_config(soma_zero::CandleCsvFormat::GenericOhlcv, true),
        )
        .expect("load valid alt fixture");
    let estimate = estimate_evidence_targets(&config, &loaded);
    assert!(estimate.estimated_outcome_records > 0);
    assert!(
        estimate
            .reason_codes
            .contains(&ReasonCode::EvidenceEstimateBuilt)
    );
}

#[test]
fn insufficient_rows_estimate_zero_and_bad_targets_remain_missing() {
    let mut config = common::onboarding_config("estimate-zero", "generic_ohlcv_valid.csv");
    config.min_rows_for_preflight = 100;
    let loader = CandleCsvLoader::default();
    let loaded = loader
        .load_from_path(
            &common::fixture_path("generic_ohlcv_valid.csv"),
            &config.build_csv_config(soma_zero::CandleCsvFormat::GenericOhlcv, true),
        )
        .expect("load valid fixture");
    let estimate = estimate_evidence_targets(&config, &loaded);
    assert_eq!(estimate.estimated_outcome_records, 0);
    assert_eq!(estimate.estimated_comparable_variants, 0);
    assert!(!estimate.enough_for_minimum_real_evidence);
}

#[test]
fn comparable_variant_estimate_respects_target_minimum() {
    let mut config = common::onboarding_config("estimate-variants", "generic_ohlcv_valid_alt.csv");
    config.target_min_comparable_variants = 2;
    config.target_min_outcomes = 1;
    let loader = CandleCsvLoader::default();
    let loaded = loader
        .load_from_path(
            &common::fixture_path("generic_ohlcv_valid_alt.csv"),
            &config.build_csv_config(soma_zero::CandleCsvFormat::GenericOhlcv, true),
        )
        .expect("load valid alt fixture");
    let estimate = estimate_evidence_targets(&config, &loaded);
    assert!(estimate.estimated_comparable_variants <= 2);
}

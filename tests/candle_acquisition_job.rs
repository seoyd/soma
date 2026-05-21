#[path = "support/candle_expansion_support.rs"]
mod candle_expansion_support;
mod common;

use soma_zero::{
    ComparableEvidenceSourceClass, OfficialCandleExpansionPlanConfig, ProviderMarket, ReasonCode,
    build_official_candle_acquisition_plan,
};

#[test]
fn acquisition_jobs_cover_local_reuse_skips_crypto_scope_budget_and_ordering() {
    candle_expansion_support::clear_env();
    let timestamps = [
        1_700_000_000_000_u64,
        1_700_086_400_000,
        1_700_172_800_000,
        1_700_259_200_000,
    ];
    let (csv, provenance, preflight, manifest) = candle_expansion_support::official_csv_fixture(
        "acq-local",
        "aapl_1d",
        "AAPL",
        "1d",
        &timestamps,
        true,
        true,
        true,
    );
    let local_gap_map = candle_expansion_support::manual_gap_map_path(
        "acq-local",
        ProviderMarket::USEquity,
        "AAPL",
        "1d",
        ComparableEvidenceSourceClass::OfficialNonCrypto,
        vec![
            csv.display().to_string(),
            provenance[0].clone(),
            preflight[0].clone(),
            manifest[0].clone(),
        ],
    );
    let local_plan = build_official_candle_acquisition_plan(&OfficialCandleExpansionPlanConfig {
        plan_id: "acq-local-plan".to_string(),
        gap_map_path: Some(local_gap_map.display().to_string()),
        output_root: common::output_dir("acq-local-plan-out")
            .display()
            .to_string(),
        ..OfficialCandleExpansionPlanConfig::default()
    })
    .expect("local plan");
    assert_eq!(
        local_plan.jobs[0].job_kind,
        soma_zero::CandleAcquisitionJobKind::ExistingCanonicalCsvReuse
    );

    let missing_csv_map = candle_expansion_support::manual_gap_map_path(
        "acq-missing-csv",
        ProviderMarket::USEquity,
        "MSFT",
        "1d",
        ComparableEvidenceSourceClass::OfficialNonCrypto,
        Vec::new(),
    );
    let missing_csv_plan =
        build_official_candle_acquisition_plan(&OfficialCandleExpansionPlanConfig {
            plan_id: "acq-missing-csv-plan".to_string(),
            gap_map_path: Some(missing_csv_map.display().to_string()),
            output_root: common::output_dir("acq-missing-csv-plan-out")
                .display()
                .to_string(),
            ..OfficialCandleExpansionPlanConfig::default()
        })
        .expect("missing csv plan");
    assert_eq!(
        missing_csv_plan.jobs[0].job_kind,
        soma_zero::CandleAcquisitionJobKind::LocalOfficialCsvImport
    );

    let krx_map = candle_expansion_support::manual_gap_map_path(
        "acq-krx",
        ProviderMarket::KoreanEquity,
        "005930",
        "1d",
        ComparableEvidenceSourceClass::OfficialNonCrypto,
        Vec::new(),
    );
    let krx_plan = build_official_candle_acquisition_plan(&OfficialCandleExpansionPlanConfig {
        plan_id: "acq-krx-plan".to_string(),
        gap_map_path: Some(krx_map.display().to_string()),
        allow_local_import: false,
        output_root: common::output_dir("acq-krx-plan-out").display().to_string(),
        ..OfficialCandleExpansionPlanConfig::default()
    })
    .expect("krx plan");
    assert_eq!(
        krx_plan.jobs[0].job_kind,
        soma_zero::CandleAcquisitionJobKind::SkippedMissingApproval
    );

    unsafe {
        std::env::set_var("KRX_APPROVAL_READY", "true");
        std::env::set_var("KRX_API_KEY", "present");
    }
    let krx_endpoint_plan =
        build_official_candle_acquisition_plan(&OfficialCandleExpansionPlanConfig {
            plan_id: "acq-krx-endpoint-plan".to_string(),
            gap_map_path: Some(krx_map.display().to_string()),
            allow_local_import: false,
            output_root: common::output_dir("acq-krx-endpoint-plan-out")
                .display()
                .to_string(),
            ..OfficialCandleExpansionPlanConfig::default()
        })
        .expect("krx endpoint plan");
    assert_eq!(
        krx_endpoint_plan.jobs[0].job_kind,
        soma_zero::CandleAcquisitionJobKind::SkippedMissingEndpointTemplate
    );
    unsafe {
        std::env::remove_var("KRX_APPROVAL_READY");
        std::env::remove_var("KRX_API_KEY");
    }

    let alpha_plan = build_official_candle_acquisition_plan(&OfficialCandleExpansionPlanConfig {
        plan_id: "acq-alpha-plan".to_string(),
        gap_map_path: Some(missing_csv_map.display().to_string()),
        allow_local_import: false,
        output_root: common::output_dir("acq-alpha-plan-out")
            .display()
            .to_string(),
        ..OfficialCandleExpansionPlanConfig::default()
    })
    .expect("alpha plan");
    assert_eq!(
        alpha_plan.jobs[0].job_kind,
        soma_zero::CandleAcquisitionJobKind::SkippedMissingAuth
    );

    let crypto_map = candle_expansion_support::manual_gap_map_path(
        "acq-crypto",
        ProviderMarket::Crypto,
        "BTCUSDT",
        "1d",
        ComparableEvidenceSourceClass::OfficialCryptoOnly,
        Vec::new(),
    );
    let crypto_plan = build_official_candle_acquisition_plan(&OfficialCandleExpansionPlanConfig {
        plan_id: "acq-crypto-plan".to_string(),
        gap_map_path: Some(crypto_map.display().to_string()),
        allow_local_import: false,
        output_root: common::output_dir("acq-crypto-plan-out")
            .display()
            .to_string(),
        ..OfficialCandleExpansionPlanConfig::default()
    })
    .expect("crypto plan");
    assert_eq!(
        crypto_plan.jobs[0].job_kind,
        soma_zero::CandleAcquisitionJobKind::UpbitCryptoCandleCollect
    );

    let ineligible_map = candle_expansion_support::manual_gap_map_path(
        "acq-ineligible",
        ProviderMarket::USEquity,
        "AAPL",
        "1d",
        ComparableEvidenceSourceClass::YFinanceResearch,
        Vec::new(),
    );
    let ineligible_plan =
        build_official_candle_acquisition_plan(&OfficialCandleExpansionPlanConfig {
            plan_id: "acq-ineligible-plan".to_string(),
            gap_map_path: Some(ineligible_map.display().to_string()),
            output_root: common::output_dir("acq-ineligible-plan-out")
                .display()
                .to_string(),
            ..OfficialCandleExpansionPlanConfig::default()
        })
        .expect("ineligible plan");
    assert_eq!(
        ineligible_plan.jobs[0].job_kind,
        soma_zero::CandleAcquisitionJobKind::SkippedSourceNotEligible
    );

    let budget_plan = build_official_candle_acquisition_plan(&OfficialCandleExpansionPlanConfig {
        plan_id: "acq-budget-plan".to_string(),
        gap_map_path: Some(missing_csv_map.display().to_string()),
        max_total_bytes: 1,
        output_root: common::output_dir("acq-budget-plan-out")
            .display()
            .to_string(),
        ..OfficialCandleExpansionPlanConfig::default()
    })
    .expect("budget plan");
    assert_eq!(
        budget_plan.jobs[0].job_kind,
        soma_zero::CandleAcquisitionJobKind::SkippedBudgetExceeded
    );
    assert!(budget_plan.storage_budget_summary.budget_exceeded);
    assert!(
        budget_plan
            .reason_codes
            .contains(&ReasonCode::OfficialEvidenceAcquisitionRan)
    );

    let first = build_official_candle_acquisition_plan(&OfficialCandleExpansionPlanConfig {
        plan_id: "acq-order-plan-1".to_string(),
        gap_map_path: Some(local_gap_map.display().to_string()),
        output_root: common::output_dir("acq-order-plan-out-1")
            .display()
            .to_string(),
        ..OfficialCandleExpansionPlanConfig::default()
    })
    .expect("first plan");
    let second = build_official_candle_acquisition_plan(&OfficialCandleExpansionPlanConfig {
        plan_id: "acq-order-plan-2".to_string(),
        gap_map_path: Some(local_gap_map.display().to_string()),
        output_root: common::output_dir("acq-order-plan-out-2")
            .display()
            .to_string(),
        ..OfficialCandleExpansionPlanConfig::default()
    })
    .expect("second plan");
    assert_eq!(first.jobs[0].job_kind, second.jobs[0].job_kind);
}

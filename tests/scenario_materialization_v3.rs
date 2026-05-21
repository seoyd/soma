mod common;
#[path = "support/sprint45_support.rs"]
mod sprint45_support;

use std::collections::BTreeMap;

use soma_zero::{
    ComparableEvidenceSourceClass, OfficialReadyRowCompletenessStatus,
    ScenarioMaterializationV3Config, ScenarioMaterializationV3Level,
    ScenarioMaterializationV3Runner,
};

#[test]
fn reuses_existing_row_level_inventory_first() {
    let inventory = sprint45_support::inventory_report(vec![sprint45_support::inventory_item("a")]);
    let report = ScenarioMaterializationV3Runner::default()
        .run_from_inventory(
            &ScenarioMaterializationV3Config::default(),
            &inventory,
            &BTreeMap::new(),
            &BTreeMap::new(),
        )
        .expect("materialize");
    assert_eq!(
        report.records[0].materialization_level,
        ScenarioMaterializationV3Level::ExistingRowLevelScenario
    );
}

#[test]
fn canonical_projection_requires_preflight_and_provenance() {
    let mut item = sprint45_support::inventory_item("b");
    item.scenario_row_id = None;
    item.has_scenario_row = false;
    let inventory = sprint45_support::inventory_report(vec![item.clone()]);
    let descriptor = soma_zero::OfficialCandleSeriesDescriptor {
        candle_series_id: "series-aapl-1d".to_string(),
        path: "candles/AAPL.csv".to_string(),
        provider_kind: None,
        source_kind: soma_zero::EvidenceSourceKind::OfficialApiCollected,
        source_class: soma_zero::OfficialCandleSeriesSourceClass::OfficialNonCrypto,
        market: soma_zero::ProviderMarket::USEquity,
        venue: Some("NASDAQ".to_string()),
        symbol: "AAPL".to_string(),
        normalized_symbol: "AAPL".to_string(),
        timeframe: "1d".to_string(),
        row_count: 32,
        timestamp_start_ms: 1,
        timestamp_end_ms: 64,
        has_duplicates: false,
        has_gaps: false,
        data_quality_score: Some(0.9),
        provenance_available: false,
        preflight_ready: false,
        manifest_available: true,
        timestamp_policy: None,
        adjusted_price_policy: None,
        official_readiness_eligible: true,
        benchmark_eligible: true,
        diagnostic_only: false,
        storage_bytes: 32,
        reason_codes: vec![],
    };
    let report = ScenarioMaterializationV3Runner::default()
        .run_from_inventory(
            &ScenarioMaterializationV3Config::default(),
            &inventory,
            &BTreeMap::from([(descriptor.candle_series_id.clone(), descriptor)]),
            &BTreeMap::new(),
        )
        .expect("materialize");
    assert_eq!(
        report.records[0].materialization_level,
        ScenarioMaterializationV3Level::Rejected
    );
}

#[test]
fn limited_feature_projection_is_reason_coded_and_no_lookahead_rejected() {
    let mut item = sprint45_support::inventory_item("c");
    item.scenario_row_id = None;
    item.has_scenario_row = false;
    item.row_level = false;
    item.summary_derived = true;
    let descriptor = soma_zero::OfficialCandleSeriesDescriptor {
        candle_series_id: "series-aapl-1d".to_string(),
        path: "candles/AAPL.csv".to_string(),
        provider_kind: None,
        source_kind: soma_zero::EvidenceSourceKind::OfficialApiCollected,
        source_class: soma_zero::OfficialCandleSeriesSourceClass::OfficialNonCrypto,
        market: soma_zero::ProviderMarket::USEquity,
        venue: Some("NASDAQ".to_string()),
        symbol: "AAPL".to_string(),
        normalized_symbol: "AAPL".to_string(),
        timeframe: "1d".to_string(),
        row_count: 32,
        timestamp_start_ms: 1,
        timestamp_end_ms: 64,
        has_duplicates: false,
        has_gaps: false,
        data_quality_score: Some(0.9),
        provenance_available: true,
        preflight_ready: true,
        manifest_available: true,
        timestamp_policy: None,
        adjusted_price_policy: None,
        official_readiness_eligible: true,
        benchmark_eligible: true,
        diagnostic_only: false,
        storage_bytes: 32,
        reason_codes: vec![],
    };
    let inventory = sprint45_support::inventory_report(vec![item.clone()]);
    let report = ScenarioMaterializationV3Runner::default()
        .run_from_inventory(
            &ScenarioMaterializationV3Config::default(),
            &inventory,
            &BTreeMap::from([(descriptor.candle_series_id.clone(), descriptor)]),
            &BTreeMap::new(),
        )
        .expect("materialize");
    assert_eq!(
        report.records[0].materialization_level,
        ScenarioMaterializationV3Level::LimitedFeatureProjected
    );

    let mut unsafe_item = item;
    unsafe_item.no_lookahead_safe = false;
    unsafe_item.completeness_statuses =
        vec![OfficialReadyRowCompletenessStatus::NoLookaheadViolation];
    let report = ScenarioMaterializationV3Runner::default()
        .run_from_inventory(
            &ScenarioMaterializationV3Config::default(),
            &sprint45_support::inventory_report(vec![unsafe_item]),
            &BTreeMap::new(),
            &BTreeMap::new(),
        )
        .expect("unsafe report");
    assert_eq!(
        report.records[0].materialization_level,
        ScenarioMaterializationV3Level::Rejected
    );
}

#[test]
fn source_class_is_not_promoted() {
    let mut item = sprint45_support::inventory_item("d");
    item.source_class = ComparableEvidenceSourceClass::ControlledDiagnostic;
    item.completeness_statuses = vec![OfficialReadyRowCompletenessStatus::DiagnosticOnly];
    let report = ScenarioMaterializationV3Runner::default()
        .run_from_inventory(
            &ScenarioMaterializationV3Config::default(),
            &sprint45_support::inventory_report(vec![item]),
            &BTreeMap::new(),
            &BTreeMap::new(),
        )
        .expect("materialize");
    assert_eq!(
        report.records[0].source_class,
        ComparableEvidenceSourceClass::ControlledDiagnostic
    );
}

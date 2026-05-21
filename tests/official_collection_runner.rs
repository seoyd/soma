mod common;

use std::fs;
use std::path::PathBuf;

use soma_zero::{
    AssetClass, CollectionOutputSize, CompressionPolicy, MarketVenue, OfficialCollectionEntry,
    OfficialCollectionEntryStatus, OfficialCollectionPlan, OfficialCollectionRunner, ProviderKind,
    RetentionPolicy, StorageBudget, Timeframe,
};

fn provider_fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("provider")
        .join(name)
}

fn example_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join(name)
}

fn mock_fixture_entry(entry_id: &str) -> OfficialCollectionEntry {
    OfficialCollectionEntry {
        entry_id: entry_id.to_string(),
        provider_kind: ProviderKind::MockFixture,
        symbol: "AAPL".to_string(),
        normalized_symbol: None,
        venue: Some(MarketVenue::NASDAQ),
        asset_class: AssetClass::Equity,
        timeframe: Timeframe::OneDay,
        start: None,
        end: None,
        max_rows: Some(100),
        max_requests: Some(1),
        outputsize: Some(CollectionOutputSize::Compact),
        auth_config_ref: None,
        endpoint_template: None,
        fixture_path: Some(
            provider_fixture_path("alphavantage_daily_compact_response.json")
                .display()
                .to_string(),
        ),
        enabled: true,
        tags: vec!["fixture".to_string()],
        reason_codes: vec![soma_zero::ReasonCode::DeterministicPath],
    }
}

fn base_plan(name: &str) -> OfficialCollectionPlan {
    OfficialCollectionPlan {
        plan_id: name.to_string(),
        output_root: common::output_dir(name).display().to_string(),
        max_total_bytes: 1024 * 1024,
        max_total_rows: 500,
        max_total_requests: 10,
        default_collection_size_policy: soma_zero::CollectionSizePolicy::default(),
        default_compression_policy: CompressionPolicy::default(),
        default_retention_policy: RetentionPolicy::KeepLastNFiles(3),
        storage_budget: StorageBudget::default(),
        entries: vec![mock_fixture_entry("entry-1")],
        continue_on_missing_auth: true,
        continue_on_provider_failure: true,
        reason_codes: vec![soma_zero::ReasonCode::DeterministicPath],
    }
}

#[test]
fn official_collection_plan_parses_toml_and_rejects_remote_fixture_paths() {
    let mut plan = base_plan("official-collection-parse");
    plan.entries[0].fixture_path = Some("https://example.com/fixture.json".to_string());
    let parsed = OfficialCollectionPlan::from_toml_str(
        &toml::to_string(&plan).expect("serialize official collection plan"),
    )
    .expect("parse official collection plan");
    assert!(
        parsed
            .validate_local_paths()
            .contains(&soma_zero::ReasonCode::LocalPathRejected)
    );
}

#[test]
fn official_collection_skips_missing_auth_conservatively() {
    let output_root = common::output_dir("official-collection-missing-auth");
    let report = OfficialCollectionRunner::default().run_plan(&OfficialCollectionPlan {
        plan_id: "official-collection-missing-auth".to_string(),
        output_root: output_root.display().to_string(),
        max_total_bytes: 1024 * 1024,
        max_total_rows: 500,
        max_total_requests: 10,
        default_collection_size_policy: soma_zero::CollectionSizePolicy::default(),
        default_compression_policy: CompressionPolicy::default(),
        default_retention_policy: RetentionPolicy::KeepLastNFiles(3),
        storage_budget: StorageBudget::default(),
        entries: vec![OfficialCollectionEntry {
            entry_id: "alpha-auth".to_string(),
            provider_kind: ProviderKind::AlphaVantage,
            symbol: "AAPL".to_string(),
            normalized_symbol: None,
            venue: Some(MarketVenue::NASDAQ),
            asset_class: AssetClass::Equity,
            timeframe: Timeframe::OneDay,
            start: None,
            end: None,
            max_rows: Some(100),
            max_requests: Some(1),
            outputsize: Some(CollectionOutputSize::Compact),
            auth_config_ref: None,
            endpoint_template: None,
            fixture_path: None,
            enabled: true,
            tags: Vec::new(),
            reason_codes: vec![soma_zero::ReasonCode::DeterministicPath],
        }],
        continue_on_missing_auth: true,
        continue_on_provider_failure: true,
        reason_codes: vec![soma_zero::ReasonCode::DeterministicPath],
    });

    assert_eq!(report.ready_entries_count, 0);
    assert_eq!(report.skipped_entries_count, 1);
    assert_eq!(
        report.entry_reports[0].status,
        OfficialCollectionEntryStatus::SkippedMissingAuth
    );
}

#[test]
fn official_collection_retention_deletes_raw_only_and_keeps_canonical() {
    let mut plan = base_plan("official-collection-retention");
    plan.default_retention_policy = RetentionPolicy::DeleteRawAfterCanonicalAndManifest;
    let report = OfficialCollectionRunner::default().run_plan(&plan);

    let entry = &report.entry_reports[0];
    let canonical_path = PathBuf::from(
        entry
            .canonical_csv_path
            .as_ref()
            .expect("canonical path in official collection report"),
    );
    let raw_dir = canonical_path
        .parent()
        .and_then(|path| path.parent())
        .expect("output dir")
        .join("raw");
    assert!(canonical_path.exists());
    assert!(raw_dir.exists());
    assert!(
        fs::read_dir(&raw_dir)
            .expect("read raw dir")
            .next()
            .is_none()
    );
    assert!(
        report
            .storage_budget_report
            .retention_actions
            .iter()
            .any(|value| value.contains("deleted raw archive"))
    );
}

#[test]
fn official_collection_storage_budget_report_is_deterministic() {
    let report_a =
        OfficialCollectionRunner::default().run_plan(&base_plan("official-collection-a"));
    let report_b =
        OfficialCollectionRunner::default().run_plan(&base_plan("official-collection-b"));

    assert_eq!(
        report_a.storage_budget_report.total_bytes,
        report_b.storage_budget_report.total_bytes
    );
    assert_eq!(
        report_a.storage_budget_report.raw_bytes,
        report_b.storage_budget_report.raw_bytes
    );
    assert_eq!(
        report_a.storage_budget_report.canonical_bytes,
        report_b.storage_budget_report.canonical_bytes
    );
    assert_eq!(
        report_a.storage_budget_report.manifest_bytes,
        report_b.storage_budget_report.manifest_bytes
    );
}

#[test]
fn sprint20_official_collection_examples_parse() {
    for path in [
        example_path("soma_official_collection_compact.toml"),
        example_path("soma_official_collection_crypto_only.toml"),
        example_path("soma_official_collection_equity_compact.toml"),
    ] {
        let plan = OfficialCollectionPlan::from_toml_path(&path).expect("parse example plan");
        assert!(!plan.entries.is_empty());
    }
}

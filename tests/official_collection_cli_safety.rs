mod common;

use std::fs;
use std::path::PathBuf;
use std::process::Command;

use soma_zero::{
    AssetClass, CollectionOutputSize, CompressionPolicy, MarketVenue, OfficialCollectionEntry,
    OfficialCollectionPlan, RetentionPolicy, StorageBudget, Timeframe,
};

fn provider_fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("provider")
        .join(name)
}

#[test]
fn cli_help_exposes_official_collection_commands() {
    let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .arg("--help")
        .output()
        .expect("run --help");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("collect-plan"));
    assert!(stdout.contains("evidence-run"));
    assert!(stdout.contains("collect-and-evaluate"));
}

#[test]
fn collect_plan_and_evidence_run_reject_remote_paths() {
    let collect_output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .args([
            "collect-plan",
            "--config",
            "https://example.com/official-plan.toml",
        ])
        .output()
        .expect("run collect-plan");
    assert!(!collect_output.status.success());
    assert!(
        String::from_utf8_lossy(&collect_output.stderr)
            .contains("collect-plan config path must be local")
    );

    let evidence_output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .args([
            "evidence-run",
            "--from-collection",
            "https://example.com/report.json",
            "--out",
            "target/official-evidence-cli",
        ])
        .output()
        .expect("run evidence-run");
    assert!(!evidence_output.status.success());
    assert!(
        String::from_utf8_lossy(&evidence_output.stderr)
            .contains("evidence-run paths must be local")
    );
}

#[test]
fn cli_collect_plan_supports_mock_fixture_without_network() {
    let output_root = common::output_dir("official-collection-cli");
    let plan_path = output_root.join("official_collection.toml");
    let plan = OfficialCollectionPlan {
        plan_id: "official-collection-cli".to_string(),
        output_root: output_root.display().to_string(),
        max_total_bytes: 1024 * 1024,
        max_total_rows: 500,
        max_total_requests: 10,
        default_collection_size_policy: soma_zero::CollectionSizePolicy::default(),
        default_compression_policy: CompressionPolicy::default(),
        default_retention_policy: RetentionPolicy::KeepLastNFiles(3),
        storage_budget: StorageBudget::default(),
        entries: vec![OfficialCollectionEntry {
            entry_id: "cli-entry".to_string(),
            provider_kind: soma_zero::ProviderKind::MockFixture,
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
        }],
        continue_on_missing_auth: true,
        continue_on_provider_failure: true,
        reason_codes: vec![soma_zero::ReasonCode::DeterministicPath],
    };
    fs::write(
        &plan_path,
        toml::to_string(&plan).expect("serialize official collection plan"),
    )
    .expect("write official collection plan");

    let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .args(["collect-plan", "--config", &plan_path.display().to_string()])
        .output()
        .expect("run collect-plan");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("plan_id=official-collection-cli"));
    assert!(stdout.contains("ready_entries_count=0"));
}

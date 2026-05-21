mod common;

use std::fs;

use soma_zero::{
    AssetClass, ConfigGenerationPolicy, MarketVenue, Timeframe, YFinanceImportConfig,
    run_yfinance_preflight_bridge,
};

fn write_bridge_fixture(name: &str, good_quality: bool) -> YFinanceImportConfig {
    let base = common::output_dir(name);
    let canonical = base.join("input.csv");
    let provenance = base.join("input.provenance.json");
    let manifest = base.join("input.manifest.json");

    let mut csv = "timestamp_ms,open,high,low,close,volume\n".to_string();
    for i in 0..48u64 {
        let price = if good_quality { 100.0 + i as f64 } else { 0.0 };
        csv.push_str(&format!(
            "{},{price:.2},{:.2},{:.2},{:.2},{}\n",
            1_704_067_200_000u64 + i * 86_400_000,
            price + 1.0,
            price.max(0.0),
            price + 0.5,
            1_000 + i
        ));
    }
    fs::write(&canonical, csv).expect("write canonical");
    fs::write(
        &provenance,
        format!(
            "{{\"source_kind\":\"YFinanceResearch\",\"source_label\":\"{name}\",\"provider_label\":\"yfinance\",\"upstream_label\":\"Yahoo Finance\",\"local_path\":\"{}\",\"generated_by\":\"test\",\"user_supplied\":true,\"downloaded_by_soma\":false,\"remote_url_present\":false,\"official_provider\":false,\"affiliated_or_endorsed\":false,\"intended_use\":\"research-only unofficial supplemental benchmark data\",\"readiness_eligible\":false,\"benchmark_eligible\":true,\"reason_codes\":[\"YFinanceCanonicalized\"]}}",
            canonical.display()
        ),
    )
    .expect("write provenance");
    fs::write(
        &manifest,
        format!(
            "{{\"manifest_version\":1,\"source_kind\":\"YFinanceResearch\",\"provider_label\":\"yfinance\",\"upstream_label\":\"Yahoo Finance\",\"symbol\":\"AAPL\",\"interval\":\"1d\",\"row_count\":48,\"first_timestamp_ms\":1704067200000,\"last_timestamp_ms\":1708137600000,\"adjusted_price_policy\":\"adjusted\",\"corporate_action_adjusted\":true,\"canonical_csv\":\"{}\",\"provenance_path\":\"{}\",\"readiness_eligible\":false,\"benchmark_eligible\":{},\"reason_codes\":[\"YFinanceCanonicalized\"]}}",
            canonical.display(),
            provenance.display(),
            if good_quality { "true" } else { "false" }
        ),
    )
    .expect("write manifest");

    YFinanceImportConfig {
        import_id: name.to_string(),
        canonical_csv_path: canonical.display().to_string(),
        output_root: base.join("out").display().to_string(),
        symbol: "AAPL".to_string(),
        venue: MarketVenue::NASDAQ,
        asset_class: AssetClass::Equity,
        timeframe: Timeframe::OneDay,
        provenance_path: Some(provenance.display().to_string()),
        manifest_path: Some(manifest.display().to_string()),
        source_label: Some(format!("yfinance-{name}")),
        config_generation_policy: ConfigGenerationPolicy::DiagnosticOnly,
        reason_codes: vec![soma_zero::ReasonCode::DeterministicPath],
    }
}

#[test]
fn yfinance_bridge_builds_research_only_preflight_report() {
    let report = run_yfinance_preflight_bridge(&write_bridge_fixture("yfinance-bridge-good", true))
        .expect("run bridge");

    assert_eq!(
        report.source_kind,
        soma_zero::EvidenceSourceKind::YFinanceResearch
    );
    assert!(report.benchmark_eligible);
    assert!(!report.official_readiness_eligible);
    assert_eq!(
        report.preflight_report.provenance.source_kind,
        soma_zero::EvidenceSourceKind::YFinanceResearch
    );
    assert!(!report.generated_config_paths.is_empty());
}

#[test]
fn low_quality_yfinance_stays_unofficial_and_not_ready() {
    let report = run_yfinance_preflight_bridge(&write_bridge_fixture("yfinance-bridge-bad", false))
        .expect("run bridge");

    assert!(!report.official_readiness_eligible);
    assert_eq!(
        report.preflight_report.provenance.readiness_eligible,
        Some(false)
    );
    assert_eq!(
        report.preflight_report.provenance.official_provider,
        Some(false)
    );
}

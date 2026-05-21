mod common;

use std::fs;

use soma_zero::{
    AssetClass, ConfigGenerationPolicy, MarketVenue, Timeframe, YFinanceImportConfig,
    YahooResearchEvidenceConfig, YahooResearchEvidenceRunner,
};

fn write_import(base_name: &str, symbol: &str) -> YFinanceImportConfig {
    let base = common::output_dir(base_name);
    let canonical = base.join(format!("{symbol}.csv"));
    let provenance = base.join(format!("{symbol}.provenance.json"));
    let manifest = base.join(format!("{symbol}.manifest.json"));

    let mut csv = "timestamp_ms,open,high,low,close,volume\n".to_string();
    for i in 0..42u64 {
        let price = 100.0 + i as f64;
        csv.push_str(&format!(
            "{},{price:.2},{:.2},{:.2},{:.2},{}\n",
            1_704_067_200_000u64 + i * 86_400_000,
            price + 1.0,
            price - 1.0,
            price + 0.5,
            1_000 + i
        ));
    }
    fs::write(&canonical, csv).expect("write csv");
    fs::write(
        &provenance,
        format!(
            "{{\"source_kind\":\"YFinanceResearch\",\"source_label\":\"{symbol}\",\"provider_label\":\"yfinance\",\"upstream_label\":\"Yahoo Finance\",\"local_path\":\"{}\",\"generated_by\":\"test\",\"user_supplied\":true,\"downloaded_by_soma\":false,\"remote_url_present\":false,\"official_provider\":false,\"affiliated_or_endorsed\":false,\"intended_use\":\"research-only unofficial supplemental benchmark data\",\"readiness_eligible\":false,\"benchmark_eligible\":true,\"reason_codes\":[\"YFinanceCanonicalized\"]}}",
            canonical.display()
        ),
    )
    .expect("write provenance");
    fs::write(
        &manifest,
        format!(
            "{{\"manifest_version\":1,\"source_kind\":\"YFinanceResearch\",\"provider_label\":\"yfinance\",\"upstream_label\":\"Yahoo Finance\",\"symbol\":\"{symbol}\",\"interval\":\"1d\",\"row_count\":42,\"first_timestamp_ms\":1704067200000,\"last_timestamp_ms\":1707600000000,\"adjusted_price_policy\":\"adjusted\",\"corporate_action_adjusted\":true,\"canonical_csv\":\"{}\",\"provenance_path\":\"{}\",\"readiness_eligible\":false,\"benchmark_eligible\":true,\"reason_codes\":[\"YFinanceCanonicalized\"]}}",
            canonical.display(),
            provenance.display()
        ),
    )
    .expect("write manifest");

    YFinanceImportConfig {
        import_id: format!("{symbol}_import"),
        canonical_csv_path: canonical.display().to_string(),
        output_root: base.join("out").display().to_string(),
        symbol: symbol.to_string(),
        venue: MarketVenue::NASDAQ,
        asset_class: AssetClass::Equity,
        timeframe: Timeframe::OneDay,
        provenance_path: Some(provenance.display().to_string()),
        manifest_path: Some(manifest.display().to_string()),
        source_label: Some(format!("yfinance-{symbol}")),
        config_generation_policy: ConfigGenerationPolicy::DiagnosticOnly,
        reason_codes: vec![soma_zero::ReasonCode::DeterministicPath],
    }
}

#[test]
fn yahoo_research_report_aggregates_imports_without_official_claims() {
    let config = YahooResearchEvidenceConfig {
        research_id: "yahoo-research-report".to_string(),
        output_root: common::output_dir("yahoo-research-root")
            .display()
            .to_string(),
        imports: vec![
            write_import("yahoo-aapl", "AAPL"),
            write_import("yahoo-msft", "MSFT"),
        ],
        reason_codes: vec![soma_zero::ReasonCode::DeterministicPath],
    };

    let report = YahooResearchEvidenceRunner::default()
        .run(&config)
        .expect("run report");

    assert_eq!(report.yfinance_symbols.len(), 2);
    assert_eq!(report.official_readiness_eligible_count, 0);
    assert_eq!(report.benchmark_eligible_count, 2);
    assert!(!report.generated_config_paths.is_empty());
}

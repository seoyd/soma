mod common;

use std::fs;
use std::process::Command;

use soma_zero::{AssetClass, ConfigGenerationPolicy, MarketVenue, Timeframe, YFinanceImportConfig};

fn write_cli_config(name: &str) -> YFinanceImportConfig {
    let base = common::output_dir(name);
    let canonical = base.join("input.csv");
    let provenance = base.join("input.provenance.json");
    let manifest = base.join("input.manifest.json");

    let mut csv = "timestamp_ms,open,high,low,close,volume\n".to_string();
    for i in 0..40u64 {
        let price = 150.0 + i as f64;
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
            "{{\"source_kind\":\"YFinanceResearch\",\"source_label\":\"{name}\",\"provider_label\":\"yfinance\",\"upstream_label\":\"Yahoo Finance\",\"local_path\":\"{}\",\"generated_by\":\"test\",\"user_supplied\":true,\"downloaded_by_soma\":false,\"remote_url_present\":false,\"official_provider\":false,\"affiliated_or_endorsed\":false,\"intended_use\":\"research-only unofficial supplemental benchmark data\",\"readiness_eligible\":false,\"benchmark_eligible\":true,\"reason_codes\":[\"YFinanceCanonicalized\"]}}",
            canonical.display()
        ),
    )
    .expect("write provenance");
    fs::write(
        &manifest,
        format!(
            "{{\"manifest_version\":1,\"source_kind\":\"YFinanceResearch\",\"provider_label\":\"yfinance\",\"upstream_label\":\"Yahoo Finance\",\"symbol\":\"AAPL\",\"interval\":\"1d\",\"row_count\":40,\"first_timestamp_ms\":1704067200000,\"last_timestamp_ms\":1707436800000,\"adjusted_price_policy\":\"adjusted\",\"corporate_action_adjusted\":true,\"canonical_csv\":\"{}\",\"provenance_path\":\"{}\",\"readiness_eligible\":false,\"benchmark_eligible\":true,\"reason_codes\":[\"YFinanceCanonicalized\"]}}",
            canonical.display(),
            provenance.display()
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
fn cli_help_mentions_yfinance_commands_as_research_only() {
    let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .arg("--help")
        .output()
        .expect("run help");
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("yfinance-import"));
    assert!(stdout.contains("yahoo-research"));
    assert!(stdout.contains("official-vs-yfinance"));
    assert!(stdout.contains("Research-only"));
}

#[test]
fn yfinance_import_rejects_remote_config_paths() {
    let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .args([
            "yfinance-import",
            "--config",
            "https://example.com/yfinance.toml",
        ])
        .output()
        .expect("run cli");
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("yfinance-import config path must be local")
    );
}

#[test]
fn yfinance_import_runs_with_local_config() {
    let config = write_cli_config("yfinance-cli-local");
    let config_path = common::output_dir("yfinance-cli-config").join("config.toml");
    fs::write(
        &config_path,
        toml::to_string_pretty(&config).expect("serialize"),
    )
    .expect("write config");

    let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .args([
            "yfinance-import",
            "--config",
            &config_path.display().to_string(),
        ])
        .output()
        .expect("run cli");
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("source_kind=YFinanceResearch"));
}

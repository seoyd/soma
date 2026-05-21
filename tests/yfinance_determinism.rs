mod common;

use std::fs;
use std::path::PathBuf;
use std::process::Command;

use soma_zero::{YFinanceImportConfig, YahooResearchEvidenceConfig};

fn research_fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("research")
        .join("fixtures")
        .join(name)
}

fn example_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join(name)
}

fn python_bin() -> Option<&'static str> {
    if Command::new("python3").arg("--version").output().is_ok() {
        Some("python3")
    } else if Command::new("python").arg("--version").output().is_ok() {
        Some("python")
    } else {
        None
    }
}

#[test]
fn sprint27_examples_parse() {
    let import = YFinanceImportConfig::from_toml_path(&example_path("soma_yfinance_import.toml"))
        .expect("parse import example");
    let report = YahooResearchEvidenceConfig::from_toml_path(&example_path(
        "soma_yfinance_research_benchmark.toml",
    ))
    .expect("parse research example");
    assert!(!import.import_id.is_empty());
    assert_eq!(report.imports.len(), 2);
}

#[test]
fn fixture_python_output_is_deterministic_when_python_available() {
    let Some(python) = python_bin() else {
        return;
    };
    let out_a = common::output_dir("yfinance-python-a");
    let out_b = common::output_dir("yfinance-python-b");
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("research")
        .join("yfinance_fetch.py");

    for out in [&out_a, &out_b] {
        let output = Command::new(python)
            .args([
                script.to_str().expect("script path"),
                "--fixture",
                research_fixture("yfinance_aapl_daily.csv")
                    .to_str()
                    .expect("fixture path"),
                "--out",
                out.to_str().expect("out path"),
                "--tickers",
                "AAPL",
                "--interval",
                "1d",
                "--period",
                "1mo",
            ])
            .output()
            .expect("run python fixture");
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let csv_a = fs::read_to_string(out_a.join("canonical").join("aapl_1d.csv")).expect("read a");
    let csv_b = fs::read_to_string(out_b.join("canonical").join("aapl_1d.csv")).expect("read b");
    let provenance =
        fs::read_to_string(out_a.join("provenance").join("aapl_1d.provenance.json")).expect("prov");
    let manifest = fs::read_to_string(out_a.join("manifests").join("aapl_1d.manifest.json"))
        .expect("manifest");

    assert_eq!(csv_a, csv_b);
    assert!(csv_a.starts_with("timestamp_ms,open,high,low,close,volume"));
    assert!(provenance.contains("\"source_kind\": \"YFinanceResearch\""));
    assert!(manifest.contains("\"benchmark_eligible\": true"));
}

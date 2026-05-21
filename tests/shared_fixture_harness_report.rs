mod support;

use std::fs;

use serde_json::json;
use support::{shared_fixture_harness as harness, sprint69_support as sprint};

#[test]
fn shared_fixture_harness_loaders_and_assertions_work() {
    let dir = harness::temp_output_dir_for_test("shared-harness-loaders");
    let json_path = dir.join("fixture.json");
    let toml_path = dir.join("fixture.toml");
    let csv_path = dir.join("fixture.csv");

    fs::write(
        &json_path,
        serde_json::to_string_pretty(&json!({"ok": true})).unwrap(),
    )
    .unwrap();
    fs::write(&toml_path, "name = \"fixture\"\nvalue = 3\n").unwrap();
    fs::write(&csv_path, "symbol,timeframe\nAAPL,1d\n").unwrap();

    let json_value: serde_json::Value = harness::load_json_fixture(&json_path);
    let toml_value: toml::Value = harness::load_toml_fixture(&toml_path);
    let csv_rows = harness::load_csv_fixture(&csv_path);

    assert_eq!(json_value["ok"], json!(true));
    assert_eq!(toml_value["name"].as_str(), Some("fixture"));
    assert_eq!(csv_rows.len(), 1);
    assert_eq!(csv_rows[0].get("symbol").map(String::as_str), Some("AAPL"));

    harness::assert_deterministic_text("same", "same");
    harness::assert_no_secret_like_values("deterministic output only");
    harness::assert_no_order_account_fields("research-only summary");
    harness::assert_no_runtime_fields("runtime deferred and training deferred");
    harness::assert_source_boundary_preserved(&json!({
        "fixtures": [{"source_boundary_fields_present": true}]
    }));
    harness::assert_no_lookahead_preserved(&json!({
        "fixtures": [{"no_lookahead_fields_present": true}]
    }));
}

#[test]
fn shared_fixture_harness_report_is_ready_and_deterministic() {
    let first = sprint::run_sprint84_bundle(
        "soma_shared_fixture_harness_report.toml",
        "sprint84-harness-report-a",
    );
    let second = sprint::run_sprint84_bundle(
        "soma_shared_fixture_harness_report.toml",
        "sprint84-harness-report-b",
    );
    assert_eq!(
        first.shared_fixture_harness_report,
        second.shared_fixture_harness_report
    );
    assert!(
        first
            .shared_fixture_harness_report
            .helper_functions_added
            .contains(&"load_json_fixture".to_string())
    );
    assert!(
        first
            .shared_fixture_harness_report
            .deterministic_output_preserved
    );
    assert!(first.shared_fixture_harness_report.secret_scan_preserved);
}

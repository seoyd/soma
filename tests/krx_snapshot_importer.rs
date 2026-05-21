mod common;

use std::fs;
use std::process::Command;

use soma_zero::{KrxSnapshotImportConfig, KrxSnapshotImporter, PreflightFinalStatus, ReasonCode};

const KRX_SNAPSHOT: &str = "종목코드,종목명,시장구분,소속부,종가,대비,등락률,시가,고가,저가,거래량,거래대금,시가총액,상장주식수\n\"060310\",\"3S\",\"KOSDAQ\",\"벤처기업부\",\"1278\",\"63\",\"5.19\",\"1217\",\"1299\",\"1217\",\"272764\",\"348197883\",\"67809453120\",\"53059040\"\n";

#[test]
fn imports_cp949_snapshot_and_writes_canonical_csv() {
    let input = common::write_cp949_temp_csv("krx_snapshot_20260510", KRX_SNAPSHOT);
    let output = common::output_dir("krx-import-output");
    let report = KrxSnapshotImporter::default()
        .import(&KrxSnapshotImportConfig {
            import_id: "krx-import".to_string(),
            input_path: input.display().to_string(),
            output_root: output.display().to_string(),
            snapshot_date: None,
            symbol_filter: Some("060310".to_string()),
            reason_codes: vec![ReasonCode::DeterministicPath],
        })
        .expect("import snapshot");

    assert_eq!(report.snapshot_date, "20260510");
    assert!(
        report
            .reason_codes
            .contains(&ReasonCode::SnapshotEncodingFallback)
    );
    let canonical = output.join("060310_krx_1d.csv");
    let contents = fs::read_to_string(canonical).expect("read canonical csv");
    assert!(contents.contains("timestamp_ms,open,high,low,close,volume,trade_value"));
    assert!(contents.contains(",1217,1299,1217,1278,272764,348197883,060310,"));
}

#[test]
fn merges_existing_rows_by_timestamp() {
    let input = common::write_cp949_temp_csv("krx_snapshot_20260510_merge", KRX_SNAPSHOT);
    let output = common::output_dir("krx-import-merge");
    let importer = KrxSnapshotImporter::default();
    let config = KrxSnapshotImportConfig {
        import_id: "krx-import-merge".to_string(),
        input_path: input.display().to_string(),
        output_root: output.display().to_string(),
        snapshot_date: None,
        symbol_filter: Some("060310".to_string()),
        reason_codes: vec![ReasonCode::DeterministicPath],
    };
    importer.import(&config).expect("first import");
    importer.import(&config).expect("second import");
    let contents = fs::read_to_string(output.join("060310_krx_1d.csv")).expect("read merged");
    assert_eq!(contents.lines().count(), 2);
}

#[test]
fn imported_canonical_csv_becomes_preflight_compatible() {
    let input = common::write_cp949_temp_csv("krx_snapshot_20260510_preflight", KRX_SNAPSHOT);
    let output = common::output_dir("krx-import-preflight");
    KrxSnapshotImporter::default()
        .import(&KrxSnapshotImportConfig {
            import_id: "krx-import-preflight".to_string(),
            input_path: input.display().to_string(),
            output_root: output.display().to_string(),
            snapshot_date: None,
            symbol_filter: Some("060310".to_string()),
            reason_codes: vec![ReasonCode::DeterministicPath],
        })
        .expect("import snapshot");

    let mut onboarding = common::onboarding_config("krx-preflight", "generic_ohlcv_valid.csv");
    onboarding.input_path = output.join("060310_krx_1d.csv").display().to_string();
    onboarding.symbol = Some("060310".to_string());
    onboarding.timeframe = Some(soma_zero::Timeframe::OneDay);
    onboarding.min_rows_for_preflight = 2;
    let report = common::run_preflight(&onboarding);
    assert_eq!(report.final_status, PreflightFinalStatus::NeedsMoreRows);
}

#[test]
fn cli_import_krx_snapshot_rejects_remote_paths_and_exposes_help() {
    let help = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .arg("--help")
        .output()
        .expect("run --help");
    assert!(help.status.success());
    let stdout = String::from_utf8_lossy(&help.stdout);
    assert!(stdout.contains("import-krx-snapshot"));

    let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .args([
            "import-krx-snapshot",
            "--input",
            "https://example.com/snapshot.csv",
            "--out",
            "target/krx-import-cli",
        ])
        .output()
        .expect("run import-krx-snapshot");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("import-krx-snapshot paths must be local"));
}

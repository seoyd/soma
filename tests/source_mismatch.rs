mod common;

use std::fs;

use soma_zero::{
    EvidenceSourceKind, SourceDatasetRecord, SourceMismatchSeverity,
    build_source_mismatch_aggregate, build_source_mismatch_report, build_source_overlap_report,
};

fn write_csv(path: &std::path::Path, rows: &[(u64, f64, f64)]) {
    let mut text = "timestamp_ms,open,high,low,close,volume\n".to_string();
    for (ts, close, volume) in rows {
        text.push_str(&format!("{ts},{close},{close},{close},{close},{volume}\n"));
    }
    fs::write(path, text).expect("write csv");
}

fn record(id: &str, kind: EvidenceSourceKind, path: &std::path::Path) -> SourceDatasetRecord {
    SourceDatasetRecord {
        dataset_id: id.to_string(),
        source_kind: kind,
        symbol: "AAPL".to_string(),
        normalized_symbol: "AAPL".to_string(),
        timeframe_label: "OneDay".to_string(),
        venue: None,
        canonical_csv_path: Some(path.display().to_string()),
        manifest_path: None,
        provenance_path: None,
        row_count: 3,
        ready_for_evidence: kind == EvidenceSourceKind::OfficialApiCollected,
        benchmark_eligible: kind == EvidenceSourceKind::YFinanceResearch,
        adjusted_price_policy: Some("raw".to_string()),
        data_quality_score: Some(1.0),
        reason_codes: vec![],
    }
}

#[test]
fn identical_fixture_data_is_none_severity() {
    let dir = common::output_dir("source-mismatch-none");
    let official = dir.join("official.csv");
    let yfinance = dir.join("yfinance.csv");
    let rows = vec![(1, 100.0, 10.0), (2, 101.0, 11.0), (3, 102.0, 12.0)];
    write_csv(&official, &rows);
    write_csv(&yfinance, &rows);
    let report = build_source_mismatch_report(
        soma_zero::SourceOverlapKey {
            normalized_symbol: "AAPL".to_string(),
            timeframe_label: "OneDay".to_string(),
            date_range_bucket: None,
        },
        &record(
            "official",
            EvidenceSourceKind::OfficialApiCollected,
            &official,
        ),
        &record("yfinance", EvidenceSourceKind::YFinanceResearch, &yfinance),
        50.0,
    )
    .expect("report");
    assert_eq!(report.severity, SourceMismatchSeverity::None);
}

#[test]
fn small_price_drift_is_low_severity() {
    let dir = common::output_dir("source-mismatch-low");
    let official = dir.join("official.csv");
    let yfinance = dir.join("yfinance.csv");
    write_csv(
        &official,
        &[(1, 100.0, 10.0), (2, 101.0, 11.0), (3, 102.0, 12.0)],
    );
    write_csv(
        &yfinance,
        &[(1, 100.1, 10.0), (2, 101.1, 11.0), (3, 102.1, 12.0)],
    );
    let report = build_source_mismatch_report(
        soma_zero::SourceOverlapKey {
            normalized_symbol: "AAPL".to_string(),
            timeframe_label: "OneDay".to_string(),
            date_range_bucket: None,
        },
        &record(
            "official",
            EvidenceSourceKind::OfficialApiCollected,
            &official,
        ),
        &record("yfinance", EvidenceSourceKind::YFinanceResearch, &yfinance),
        50.0,
    )
    .expect("report");
    assert_eq!(report.severity, SourceMismatchSeverity::Low);
}

#[test]
fn large_price_drift_is_high_severity() {
    let dir = common::output_dir("source-mismatch-high");
    let official = dir.join("official.csv");
    let yfinance = dir.join("yfinance.csv");
    write_csv(
        &official,
        &[(1, 100.0, 10.0), (2, 101.0, 11.0), (3, 102.0, 12.0)],
    );
    write_csv(
        &yfinance,
        &[(1, 120.0, 10.0), (2, 121.0, 11.0), (3, 122.0, 12.0)],
    );
    let report = build_source_mismatch_report(
        soma_zero::SourceOverlapKey {
            normalized_symbol: "AAPL".to_string(),
            timeframe_label: "OneDay".to_string(),
            date_range_bucket: None,
        },
        &record(
            "official",
            EvidenceSourceKind::OfficialApiCollected,
            &official,
        ),
        &record("yfinance", EvidenceSourceKind::YFinanceResearch, &yfinance),
        50.0,
    )
    .expect("report");
    assert_eq!(report.severity, SourceMismatchSeverity::High);
}

#[test]
fn mismatch_aggregate_is_deterministic() {
    let dir = common::output_dir("source-mismatch-agg");
    let official = dir.join("official.csv");
    let yfinance = dir.join("yfinance.csv");
    let rows = vec![(1, 100.0, 10.0), (2, 101.0, 11.0), (3, 102.0, 12.0)];
    write_csv(&official, &rows);
    write_csv(&yfinance, &rows);
    let official_record = record(
        "official",
        EvidenceSourceKind::OfficialApiCollected,
        &official,
    );
    let yfinance_record = record("yfinance", EvidenceSourceKind::YFinanceResearch, &yfinance);
    let overlap = build_source_overlap_report(
        std::slice::from_ref(&official_record),
        std::slice::from_ref(&yfinance_record),
    );
    let first = build_source_mismatch_aggregate(
        &overlap,
        &[official_record.clone()],
        &[yfinance_record.clone()],
        50.0,
    )
    .expect("first");
    let second =
        build_source_mismatch_aggregate(&overlap, &[official_record], &[yfinance_record], 50.0)
            .expect("second");
    assert_eq!(first, second);
}

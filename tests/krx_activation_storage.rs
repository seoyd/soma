mod common;

use std::fs;

use soma_zero::KRXActivationStorageReport;

#[test]
fn storage_report_counts_bytes_and_sorts_largest_artifacts() {
    let out = common::output_dir("krx-storage");
    let canonical = out.join("canonical.csv");
    let provenance = out.join("provenance.json");
    let preflight = out.join("preflight.json");
    let downstream = out.join("bundle.json");
    fs::write(&canonical, "1234567890").expect("write canonical");
    fs::write(&provenance, "12345").expect("write provenance");
    fs::write(&preflight, "1234567").expect("write preflight");
    fs::write(&downstream, "12345678901234").expect("write downstream");

    let report = KRXActivationStorageReport::build(
        &[canonical.display().to_string()],
        &[],
        &[provenance.display().to_string()],
        &[preflight.display().to_string()],
        &[],
        &[downstream.display().to_string()],
        &[],
        8,
    );

    assert_eq!(report.canonical_csv_bytes, 10);
    assert_eq!(report.provenance_bytes, 5);
    assert_eq!(report.preflight_bytes, 7);
    assert_eq!(report.downstream_bundle_bytes, 14);
    assert!(report.budget_exceeded);
    assert_eq!(
        report.largest_artifacts[0],
        format!("{}=14", downstream.display())
    );
    assert!(
        report
            .compaction_recommendation
            .contains("canonical evidence is retained")
    );
}

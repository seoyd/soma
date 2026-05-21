mod common;

use std::fs;
use std::path::{Path, PathBuf};

use soma_zero::{
    OfficialCollectionEntryReport, OfficialCollectionEntryStatus, OfficialCollectionReport,
    OfficialEvidenceRecommendation, OfficialEvidenceRunConfig, OfficialEvidenceRunner,
    ProviderKind, StorageBudgetReport, Timeframe,
};

fn write_collection_report(
    name: &str,
    entry_reports: Vec<OfficialCollectionEntryReport>,
) -> (PathBuf, PathBuf) {
    let output_root = common::output_dir(name);
    let plan_root = output_root.join("official-plan").join("entry").join("1d");
    fs::create_dir_all(plan_root.join("canonical")).expect("create canonical dir");
    let report = OfficialCollectionReport {
        plan_id: name.to_string(),
        ready_entries_count: entry_reports
            .iter()
            .filter(|entry| entry.ready_for_evidence)
            .count(),
        skipped_entries_count: entry_reports
            .iter()
            .filter(|entry| {
                matches!(
                    entry.status,
                    OfficialCollectionEntryStatus::SkippedMissingAuth
                )
            })
            .count(),
        failed_entries_count: entry_reports
            .iter()
            .filter(|entry| matches!(entry.status, OfficialCollectionEntryStatus::FailedProvider))
            .count(),
        official_api_collected_count: entry_reports
            .iter()
            .filter(|entry| matches!(entry.status, OfficialCollectionEntryStatus::Collected))
            .count(),
        entry_reports,
        storage_budget_report: StorageBudgetReport::default(),
        reason_codes: vec![soma_zero::ReasonCode::OfficialCollectionRan],
    };
    let report_path = output_root.join("official_collection_report.json");
    fs::write(
        &report_path,
        report
            .to_json_string()
            .expect("serialize collection report"),
    )
    .expect("write collection report");
    (report_path, plan_root)
}

fn ready_entry(canonical_csv_path: &Path) -> OfficialCollectionEntryReport {
    OfficialCollectionEntryReport {
        entry_id: "ready-entry".to_string(),
        provider_kind: ProviderKind::MockFixture,
        symbol: "AAPL".to_string(),
        venue: Some(soma_zero::MarketVenue::NASDAQ),
        timeframe: Timeframe::OneDay,
        status: OfficialCollectionEntryStatus::Collected,
        canonical_csv_path: Some(canonical_csv_path.display().to_string()),
        manifest_path: None,
        provenance_path: None,
        preflight_status: Some("ReadyForRealEvidence".to_string()),
        row_count: 120,
        request_count: 1,
        bytes_written: 1024,
        compressed: false,
        ready_for_evidence: true,
        reason_codes: vec![soma_zero::ReasonCode::OfficialCollectionEntryCollected],
    }
}

fn example_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join(name)
}

#[test]
fn official_evidence_runner_uses_generated_config_from_collection_output_dir() {
    common::ensure_sprint15_report();

    let output_root = common::output_dir("official-evidence-ready");
    let collection_output_dir = output_root.join("official-plan").join("entry").join("1d");
    fs::create_dir_all(collection_output_dir.join("canonical")).expect("create canonical dir");
    let canonical_csv_path = collection_output_dir
        .join("canonical")
        .join("AAPL_1d_compact.csv");
    fs::write(
        &canonical_csv_path,
        "timestamp,open,high,low,close,volume\n",
    )
    .expect("write canonical csv");

    let mut real_config = common::real_evidence_config(
        "official-evidence-generated",
        vec![common::real_local_test_entry(
            "real-alt",
            "generic_ohlcv_valid_alt.csv",
        )],
    );
    real_config.output_root = collection_output_dir
        .join("generated-real-evidence")
        .display()
        .to_string();
    real_config.evidence_store_path = collection_output_dir
        .join("generated-evidence-store")
        .display()
        .to_string();
    real_config.min_real_local_outcome_records = 1;
    real_config.min_real_local_comparable_variants = 1;
    fs::write(
        collection_output_dir.join("generated_real_evidence_closure.toml"),
        toml::to_string(&real_config).expect("serialize real evidence config"),
    )
    .expect("write generated real evidence config");

    let report = OfficialCollectionReport {
        plan_id: "official-evidence-ready".to_string(),
        ready_entries_count: 1,
        skipped_entries_count: 0,
        failed_entries_count: 0,
        official_api_collected_count: 1,
        entry_reports: vec![ready_entry(&canonical_csv_path)],
        storage_budget_report: StorageBudgetReport::default(),
        reason_codes: vec![soma_zero::ReasonCode::OfficialCollectionRan],
    };
    let collection_report_path = output_root.join("official_collection_report.json");
    fs::write(
        &collection_report_path,
        report
            .to_json_string()
            .expect("serialize collection report"),
    )
    .expect("write collection report");

    let report = OfficialEvidenceRunner::default().run(&OfficialEvidenceRunConfig {
        collection_report_path: Some(collection_report_path.display().to_string()),
        generated_rerun_configs: Vec::new(),
        output_root: output_root
            .join("official-evidence-out")
            .display()
            .to_string(),
        run_real_evidence: true,
        run_batch: false,
        run_ablation: false,
        require_ready_entries: true,
        min_ready_entries: 1,
        min_outcome_records: 1,
        min_comparable_variants: 1,
        reason_codes: vec![soma_zero::ReasonCode::DeterministicPath],
    });

    assert!(report.outcome_records > 0);
    assert!(report.comparable_variants > 0);
    assert_ne!(
        report.recommendation,
        OfficialEvidenceRecommendation::NeedMoreExperiments
    );
}

#[test]
fn official_evidence_runner_returns_missing_auth_when_ready_entries_are_insufficient() {
    let output_root = common::output_dir("official-evidence-missing-auth");
    let collection_output_dir = output_root.join("official-plan").join("entry").join("1d");
    fs::create_dir_all(collection_output_dir.join("canonical")).expect("create canonical dir");
    let canonical_csv_path = collection_output_dir
        .join("canonical")
        .join("AAPL_1d_compact.csv");
    fs::write(
        &canonical_csv_path,
        "timestamp,open,high,low,close,volume\n",
    )
    .expect("write canonical csv");

    let report = OfficialCollectionReport {
        plan_id: "official-evidence-missing-auth".to_string(),
        ready_entries_count: 1,
        skipped_entries_count: 1,
        failed_entries_count: 0,
        official_api_collected_count: 1,
        entry_reports: vec![
            ready_entry(&canonical_csv_path),
            OfficialCollectionEntryReport {
                entry_id: "missing-auth".to_string(),
                provider_kind: ProviderKind::AlphaVantage,
                symbol: "AAPL".to_string(),
                venue: Some(soma_zero::MarketVenue::NASDAQ),
                timeframe: Timeframe::OneDay,
                status: OfficialCollectionEntryStatus::SkippedMissingAuth,
                canonical_csv_path: None,
                manifest_path: None,
                provenance_path: None,
                preflight_status: None,
                row_count: 0,
                request_count: 0,
                bytes_written: 0,
                compressed: false,
                ready_for_evidence: false,
                reason_codes: vec![
                    soma_zero::ReasonCode::OfficialCollectionEntrySkippedMissingAuth,
                ],
            },
        ],
        storage_budget_report: StorageBudgetReport::default(),
        reason_codes: vec![soma_zero::ReasonCode::OfficialCollectionRan],
    };
    let collection_report_path = output_root.join("official_collection_report.json");
    fs::write(
        &collection_report_path,
        report
            .to_json_string()
            .expect("serialize collection report"),
    )
    .expect("write collection report");

    let report = OfficialEvidenceRunner::default().run(&OfficialEvidenceRunConfig {
        collection_report_path: Some(collection_report_path.display().to_string()),
        generated_rerun_configs: Vec::new(),
        output_root: output_root
            .join("official-evidence-out")
            .display()
            .to_string(),
        run_real_evidence: false,
        run_batch: false,
        run_ablation: false,
        require_ready_entries: true,
        min_ready_entries: 2,
        min_outcome_records: 1,
        min_comparable_variants: 1,
        reason_codes: vec![soma_zero::ReasonCode::DeterministicPath],
    });

    assert_eq!(
        report.recommendation,
        OfficialEvidenceRecommendation::MissingAuth
    );
}

#[test]
fn official_evidence_runner_stays_conservative_without_enough_results() {
    let output_root = common::output_dir("official-evidence-conservative");
    let collection_output_dir = output_root.join("official-plan").join("entry").join("1d");
    fs::create_dir_all(collection_output_dir.join("canonical")).expect("create canonical dir");
    let canonical_csv_path = collection_output_dir
        .join("canonical")
        .join("AAPL_1d_compact.csv");
    fs::write(
        &canonical_csv_path,
        "timestamp,open,high,low,close,volume\n",
    )
    .expect("write canonical csv");

    let (collection_report_path, _) = write_collection_report(
        "official-evidence-conservative",
        vec![ready_entry(&canonical_csv_path)],
    );
    let report = OfficialEvidenceRunner::default().run(&OfficialEvidenceRunConfig {
        collection_report_path: Some(collection_report_path.display().to_string()),
        generated_rerun_configs: Vec::new(),
        output_root: output_root
            .join("official-evidence-out")
            .display()
            .to_string(),
        run_real_evidence: false,
        run_batch: false,
        run_ablation: false,
        require_ready_entries: true,
        min_ready_entries: 1,
        min_outcome_records: 1,
        min_comparable_variants: 1,
        reason_codes: vec![soma_zero::ReasonCode::DeterministicPath],
    });

    assert_eq!(
        report.recommendation,
        OfficialEvidenceRecommendation::NeedMoreExperiments
    );
    assert!(report.outcome_records == 0);
}

#[test]
fn sprint20_official_evidence_example_parses() {
    let config =
        OfficialEvidenceRunConfig::from_toml_path(&example_path("soma_official_evidence_run.toml"))
            .expect("parse official evidence example");
    assert!(config.run_real_evidence);
}

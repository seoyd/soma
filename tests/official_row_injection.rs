mod common;
#[path = "support/official_committee_support.rs"]
mod official_committee_support;

use std::fs;

use serde_json::json;
use soma_zero::{
    CommitteeScenarioMaterializationLevel, CommitteeScenarioSourceKind, EvidenceSourceKind,
    OfficialCommitteeScenarioPack, OfficialEvidenceReplicationConfig,
    OfficialEvidenceReplicationRunner, OfficialReplicationArtifactInventory, OfficialRowInjector,
    ReasonCode,
};

#[test]
fn row_injection_prefers_existing_pack_and_is_deterministic() {
    let mut row = official_committee_support::scenario_row(
        "row-injection-pack",
        0,
        "AAPL",
        1_700_000_000_000,
    );
    row.provenance_summary = "row-level-provenance: official-api-collected".to_string();
    row.source_kind = CommitteeScenarioSourceKind::EvidenceLaneReport;
    let pack = OfficialCommitteeScenarioPack {
        pack_id: "row-injection-pack".to_string(),
        rows: vec![row],
        source_summary: "OfficialApiCollected=1".to_string(),
        official_row_count: 1,
        crypto_only_row_count: 0,
        yfinance_row_count: 0,
        fixture_row_count: 0,
        row_level_count: 1,
        summary_derived_count: 0,
        outcome_linked_count: 0,
        baseline_reference_count: 1,
        external_reference_count: 0,
        no_trade_counterfactual_count: 0,
        risk_denial_counterfactual_count: 0,
        storage_bytes: 1,
        reason_codes: vec![ReasonCode::DeterministicPath],
    };
    let pack_dir = common::output_dir("row-injection-pack-dir");
    let pack_path = pack.write_to_dir(&pack_dir).expect("pack write");
    let lane_path = common::output_dir("row-injection-lane").join("evidence_lane_report.json");
    fs::write(
        &lane_path,
        serde_json::to_string_pretty(&json!({
            "provenance": "official-api-collected",
            "lane_reports": [{
                "symbol": "AAPL",
                "market": "USEquity",
                "timestamp_ms": 1700000000000u64,
                "data_quality_score": 0.90,
                "expected_edge_after_cost": 0.01,
                "expected_drawdown": 0.01
            }]
        }))
        .expect("lane json"),
    )
    .expect("write lane");
    let inventory = OfficialReplicationArtifactInventory::from_paths(&vec![
        pack_path.display().to_string(),
        lane_path.display().to_string(),
    ]);
    let config = OfficialEvidenceReplicationConfig {
        official_committee_pack_paths: vec![pack_path.display().to_string()],
        evidence_lane_report_paths: vec![lane_path.display().to_string()],
        require_provenance: false,
        require_preflight: false,
        ..OfficialEvidenceReplicationConfig::default()
    };
    let first = OfficialRowInjector::default()
        .inject(&config, &inventory, &config.row_injection_policy())
        .expect("first");
    let second = OfficialRowInjector::default()
        .inject(&config, &inventory, &config.row_injection_policy())
        .expect("second");
    assert_eq!(first, second);
    assert_eq!(first.injected_rows.len(), 1);
    assert_eq!(
        first.injected_rows[0].scenario_row_id,
        "row-injection-pack-0"
    );
}

#[test]
fn row_injection_uses_evidence_lane_when_allowed_and_canonical_csv_skips_missing_requirements() {
    let official_lane_path =
        common::output_dir("row-injection-official-lane").join("evidence_lane_report.json");
    fs::write(
        &official_lane_path,
        serde_json::to_string_pretty(&json!({
            "provenance": "official-api-collected",
            "lane_reports": [{
                "symbol": "AAPL",
                "market": "USEquity",
                "timestamp_ms": 1700000000000u64,
                "data_quality_score": 0.90,
                "expected_edge_after_cost": 0.01,
                "expected_drawdown": 0.01
            }]
        }))
        .expect("lane json"),
    )
    .expect("write lane");
    let missing_provenance_csv = official_committee_support::write_official_csv_bundle(
        "row-injection-missing-provenance",
        "AAPL",
        3,
        true,
        false,
        true,
    );
    let missing_preflight_csv = official_committee_support::write_official_csv_bundle(
        "row-injection-missing-preflight",
        "AAPL",
        3,
        false,
        true,
        true,
    );
    let good_csv = official_committee_support::write_official_csv_bundle(
        "row-injection-good-csv",
        "AAPL",
        3,
        true,
        true,
        true,
    );
    let inventory = OfficialReplicationArtifactInventory::from_paths(&vec![
        official_lane_path.display().to_string(),
        missing_provenance_csv.display().to_string(),
        missing_preflight_csv.display().to_string(),
        good_csv.display().to_string(),
    ]);
    let lane_only_config = OfficialEvidenceReplicationConfig {
        evidence_lane_report_paths: vec![official_lane_path.display().to_string()],
        require_provenance: false,
        require_preflight: false,
        max_rows: 1,
        output_root: common::output_dir("row-injection-out")
            .display()
            .to_string(),
        ..OfficialEvidenceReplicationConfig::default()
    };
    let lane_result = OfficialRowInjector::default()
        .inject(
            &lane_only_config,
            &inventory,
            &lane_only_config.row_injection_policy(),
        )
        .expect("inject lane");
    assert_eq!(lane_result.injected_rows.len(), 1);
    assert_eq!(lane_result.injected_rows[0].symbol, "AAPL");

    let csv_only_config = OfficialEvidenceReplicationConfig {
        official_canonical_csv_paths: vec![
            missing_provenance_csv.display().to_string(),
            missing_preflight_csv.display().to_string(),
            good_csv.display().to_string(),
        ],
        evidence_lane_report_paths: Vec::new(),
        max_rows: 1,
        output_root: common::output_dir("row-injection-csv-only")
            .display()
            .to_string(),
        ..OfficialEvidenceReplicationConfig::default()
    };
    let csv_only_result = OfficialRowInjector::default()
        .inject(
            &csv_only_config,
            &inventory,
            &csv_only_config.row_injection_policy(),
        )
        .expect("csv only");
    assert_eq!(csv_only_result.skipped_missing_provenance, 0);
    assert!(csv_only_result.skipped_missing_preflight >= 1);
    assert!(!csv_only_result.injected_rows.is_empty());
}

#[test]
fn row_injection_separates_crypto_and_skips_research_fixture_controlled_and_summary_rows() {
    let crypto_lane =
        official_committee_support::write_crypto_evidence_lane("row-injection-crypto");
    let yfinance_path =
        common::output_dir("row-injection-yfinance").join("evidence_lane_report.json");
    fs::write(
        &yfinance_path,
        serde_json::to_string_pretty(&json!({
            "provenance": "yfinance-research",
            "lane_reports": [{"symbol": "AAPL", "market": "USEquity"}]
        }))
        .expect("yfinance json"),
    )
    .expect("write yfinance");
    let fixture_path =
        common::output_dir("row-injection-fixture").join("evidence_lane_report.json");
    fs::write(
        &fixture_path,
        serde_json::to_string_pretty(&json!({
            "provenance": "fixture",
            "lane_reports": [{"symbol": "AAPL", "market": "USEquity"}]
        }))
        .expect("fixture json"),
    )
    .expect("write fixture");
    let mut controlled_row = official_committee_support::scenario_row(
        "row-injection-controlled",
        0,
        "AAPL",
        1_700_000_000_000,
    );
    controlled_row.evidence_source_kind = EvidenceSourceKind::RealLocal;
    controlled_row.provenance_summary = "controlled-local".to_string();
    controlled_row.materialization_level = CommitteeScenarioMaterializationLevel::BenchmarkSummary;
    let controlled_pack = OfficialCommitteeScenarioPack {
        pack_id: "row-injection-controlled-pack".to_string(),
        rows: vec![controlled_row],
        source_summary: "Controlled=1".to_string(),
        official_row_count: 0,
        crypto_only_row_count: 0,
        yfinance_row_count: 0,
        fixture_row_count: 0,
        row_level_count: 0,
        summary_derived_count: 1,
        outcome_linked_count: 0,
        baseline_reference_count: 0,
        external_reference_count: 0,
        no_trade_counterfactual_count: 0,
        risk_denial_counterfactual_count: 0,
        storage_bytes: 1,
        reason_codes: vec![ReasonCode::SummaryDerived],
    };
    let controlled_dir = common::output_dir("row-injection-controlled-pack-dir");
    let controlled_pack_path = controlled_pack
        .write_to_dir(&controlled_dir)
        .expect("controlled pack");

    let inventory = OfficialReplicationArtifactInventory::from_paths(&vec![
        crypto_lane.display().to_string(),
        yfinance_path.display().to_string(),
        fixture_path.display().to_string(),
        controlled_pack_path.display().to_string(),
    ]);
    let config = OfficialEvidenceReplicationConfig {
        evidence_lane_report_paths: vec![
            crypto_lane.display().to_string(),
            yfinance_path.display().to_string(),
            fixture_path.display().to_string(),
        ],
        official_committee_pack_paths: vec![controlled_pack_path.display().to_string()],
        require_provenance: false,
        require_preflight: false,
        output_root: common::output_dir("row-injection-crypto-out")
            .display()
            .to_string(),
        ..OfficialEvidenceReplicationConfig::default()
    };
    let result = OfficialRowInjector::default()
        .inject(&config, &inventory, &config.row_injection_policy())
        .expect("inject");
    assert_eq!(result.crypto_only_row_count, result.injected_rows.len());
    assert_eq!(result.non_crypto_official_row_count, 0);
    assert_eq!(result.skipped_research_only, 1);
    assert_eq!(result.skipped_fixture, 1);

    let summary_only_config = OfficialEvidenceReplicationConfig {
        official_committee_pack_paths: vec![controlled_pack_path.display().to_string()],
        require_provenance: false,
        require_preflight: false,
        ..OfficialEvidenceReplicationConfig::default()
    };
    let summary_only_result = OfficialEvidenceReplicationRunner::default()
        .row_injection(&summary_only_config)
        .expect("summary result");
    assert_eq!(summary_only_result.injected_rows.len(), 0);
    assert_eq!(summary_only_result.skipped_summary_derived, 1);
}

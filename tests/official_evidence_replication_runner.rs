mod common;
#[path = "support/official_committee_support.rs"]
mod official_committee_support;

use std::fs;

use soma_zero::{
    OfficialEvidenceReplicationConfig, OfficialEvidenceReplicationFinalStatus,
    OfficialEvidenceReplicationRecommendation, OfficialEvidenceReplicationRunner,
    OfficialProviderReadinessReport, OfficialProviderReadinessStatus, ProviderRealityReport,
    ProviderRealitySummary, ReasonCode, build_default_provider_catalog,
    default_provider_cost_profiles, default_provider_freshness_profiles,
};

fn write_readiness_report(name: &str) -> std::path::PathBuf {
    let dir = common::output_dir(&format!("{name}-runner-readiness"));
    let report = OfficialProviderReadinessReport {
        report_id: name.to_string(),
        catalog: build_default_provider_catalog(),
        credential_statuses: Vec::new(),
        selection_results: Vec::new(),
        implemented_providers: vec![],
        missing_auth_actions: vec!["alphavantage missing auth".to_string()],
        deferred_provider_actions: Vec::new(),
        official_ready_markets: Vec::new(),
        research_only_markets: Vec::new(),
        final_status: OfficialProviderReadinessStatus::MissingUSAuth,
        reason_codes: vec![ReasonCode::ProviderReadinessReportBuilt],
    };
    report.write_to_dir(&dir).expect("write readiness")
}

fn write_reality_report(name: &str) -> std::path::PathBuf {
    let dir = common::output_dir(&format!("{name}-runner-reality"));
    let report = ProviderRealityReport {
        report_id: name.to_string(),
        freshness_profiles: default_provider_freshness_profiles(),
        cost_profiles: default_provider_cost_profiles(),
        entitlement_statuses: Vec::new(),
        compatibility_results: Vec::new(),
        recommendations: Vec::new(),
        operator_actions: Vec::new(),
        final_summary: vec![ProviderRealitySummary::NeedProviderAuthSetup],
        reason_codes: vec![ReasonCode::ProviderRealityReportBuilt],
    };
    report.write_to_dir(&dir).expect("write reality")
}

#[test]
fn replication_runner_reports_missing_official_auth_before_collection() {
    let readiness = write_readiness_report("replication-auth");
    let reality = write_reality_report("replication-auth");
    let config = OfficialEvidenceReplicationConfig {
        replication_id: "replication-auth".to_string(),
        output_root: common::output_dir("replication-auth-root")
            .display()
            .to_string(),
        provider_readiness_report_paths: vec![readiness.display().to_string()],
        provider_reality_report_paths: vec![reality.display().to_string()],
        ..OfficialEvidenceReplicationConfig::default()
    };
    let report = OfficialEvidenceReplicationRunner::default()
        .run(&config)
        .expect("run replication");
    assert_eq!(
        report.final_status,
        OfficialEvidenceReplicationFinalStatus::MissingOfficialAuth
    );
    assert_eq!(
        report.final_recommendation,
        OfficialEvidenceReplicationRecommendation::SetAlphaVantageAuth
    );
}

#[test]
fn replication_runner_reports_missing_provenance_and_preflight() {
    let missing_provenance = common::output_dir("replication-missing-prov").join("AAPL.csv");
    fs::write(
        &missing_provenance,
        "timestamp,open,high,low,close,volume\n1700000000000,1,1,1,1,1\n",
    )
    .expect("write csv");
    fs::write(
        missing_provenance
            .parent()
            .expect("dir")
            .join("preflight_report.json"),
        serde_json::to_string_pretty(&serde_json::json!({
            "onboarding_id": "replication-missing-prov",
            "final_status": "ReadyForRealEvidence",
            "checks": [],
            "symbol": "AAPL",
            "output_dir": "target"
        }))
        .expect("preflight json"),
    )
    .expect("write preflight");
    let prov_config = OfficialEvidenceReplicationConfig {
        replication_id: "replication-missing-prov".to_string(),
        output_root: common::output_dir("replication-missing-prov-root")
            .display()
            .to_string(),
        official_canonical_csv_paths: vec![missing_provenance.display().to_string()],
        ..OfficialEvidenceReplicationConfig::default()
    };
    let prov_report = OfficialEvidenceReplicationRunner::default()
        .run(&prov_config)
        .expect("run missing provenance");
    assert_eq!(
        prov_report.final_status,
        OfficialEvidenceReplicationFinalStatus::MissingOfficialProvenance
    );

    let missing_preflight = common::output_dir("replication-missing-preflight").join("AAPL.csv");
    fs::write(
        &missing_preflight,
        "timestamp,open,high,low,close,volume\n1700000000000,1,1,1,1,1\n",
    )
    .expect("write csv");
    fs::write(
        missing_preflight
            .parent()
            .expect("dir")
            .join("official_provenance.json"),
        serde_json::to_string_pretty(&serde_json::json!({
            "symbol": "AAPL",
            "source_kind": "OfficialApiCollected",
            "official_provider": true,
            "downloaded_by_soma": true,
            "reason_codes": ["OfficialApiCollected"]
        }))
        .expect("prov json"),
    )
    .expect("write provenance");
    let preflight_config = OfficialEvidenceReplicationConfig {
        replication_id: "replication-missing-preflight".to_string(),
        output_root: common::output_dir("replication-missing-preflight-root")
            .display()
            .to_string(),
        official_canonical_csv_paths: vec![missing_preflight.display().to_string()],
        ..OfficialEvidenceReplicationConfig::default()
    };
    let preflight_report = OfficialEvidenceReplicationRunner::default()
        .run(&preflight_config)
        .expect("run missing preflight");
    assert_eq!(
        preflight_report.final_status,
        OfficialEvidenceReplicationFinalStatus::MissingOfficialPreflight
    );
}

#[test]
fn replication_runner_reports_missing_candles_for_pack_only_inputs() {
    let mut row = official_committee_support::scenario_row(
        "replication-missing-candles",
        0,
        "AAPL",
        1_700_000_000_000,
    );
    row.provenance_summary = "row-level-provenance: official-api-collected".to_string();
    let pack = soma_zero::OfficialCommitteeScenarioPack {
        pack_id: "replication-missing-candles-pack".to_string(),
        rows: vec![row],
        source_summary: "Official=1".to_string(),
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
    let pack_path = pack
        .write_to_dir(&common::output_dir("replication-missing-candles-pack"))
        .expect("write pack");
    let config = OfficialEvidenceReplicationConfig {
        replication_id: "replication-missing-candles".to_string(),
        output_root: common::output_dir("replication-missing-candles-root")
            .display()
            .to_string(),
        official_committee_pack_paths: vec![pack_path.display().to_string()],
        require_preflight: false,
        require_provenance: false,
        ..OfficialEvidenceReplicationConfig::default()
    };
    let report = OfficialEvidenceReplicationRunner::default()
        .run(&config)
        .expect("run pack-only");
    assert_eq!(
        report.final_status,
        OfficialEvidenceReplicationFinalStatus::MissingOfficialCandles
    );
}

#[test]
fn replication_runner_reaches_official_ready_for_complete_local_bundle() {
    let csv_path = common::output_dir("replication-complete").join("AAPL.csv");
    let mut csv = String::from("timestamp,open,high,low,close,volume\n");
    for i in 0..32u64 {
        csv.push_str(&format!(
            "{},100,101,99,100.5,1000\n",
            1_700_000_000_000u64 + i
        ));
    }
    fs::write(&csv_path, csv).expect("write csv");
    fs::write(
        csv_path
            .parent()
            .expect("dir")
            .join("preflight_report.json"),
        serde_json::to_string_pretty(&serde_json::json!({
            "onboarding_id": "replication-complete",
            "final_status": "ReadyForRealEvidence",
            "checks": [],
            "symbol": "AAPL",
            "output_dir": "target"
        }))
        .expect("preflight json"),
    )
    .expect("write preflight");
    fs::write(
        csv_path
            .parent()
            .expect("dir")
            .join("official_provenance.json"),
        serde_json::to_string_pretty(&serde_json::json!({
            "symbol": "AAPL",
            "source_kind": "OfficialApiCollected",
            "official_provider": true,
            "downloaded_by_soma": true,
            "reason_codes": ["OfficialApiCollected"]
        }))
        .expect("provenance json"),
    )
    .expect("write provenance");
    let config = OfficialEvidenceReplicationConfig {
        replication_id: "replication-complete".to_string(),
        output_root: common::output_dir("replication-complete-root")
            .display()
            .to_string(),
        official_canonical_csv_paths: vec![csv_path.display().to_string()],
        max_rows: 1,
        run_official_committee_benchmark: false,
        ..OfficialEvidenceReplicationConfig::default()
    };
    let report = OfficialEvidenceReplicationRunner::default()
        .run(&config)
        .expect("run complete");
    assert_eq!(
        report.final_status,
        OfficialEvidenceReplicationFinalStatus::OfficialReplicationReady
    );
    assert_eq!(
        report.final_recommendation,
        OfficialEvidenceReplicationRecommendation::KeepTrinity
    );
    assert!(
        report
            .official_sufficiency_replication_report
            .passed_for_official
    );
}

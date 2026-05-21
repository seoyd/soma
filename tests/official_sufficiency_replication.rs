mod common;
#[path = "support/official_committee_support.rs"]
mod official_committee_support;

use std::fs;

use soma_zero::{
    OfficialReferenceReplicationArtifacts, OfficialReferenceReplicationReport,
    OfficialSufficiencyReplicationBuilder, OfficialSufficiencyReplicationStatus,
};

fn row(name: &str) -> soma_zero::CommitteeScenarioRow {
    let mut row = official_committee_support::scenario_row(name, 0, "AAPL", 1_700_000_000_000);
    row.provenance_summary = "row-level-provenance: official-api-collected".to_string();
    row
}

fn injection(
    rows: Vec<soma_zero::CommitteeScenarioRow>,
    official_row_count: usize,
    non_crypto_official_row_count: usize,
    crypto_only_row_count: usize,
) -> soma_zero::OfficialRowInjectionResult {
    soma_zero::OfficialRowInjectionResult {
        injected_rows: rows,
        skipped_rows: Vec::new(),
        official_row_count,
        non_crypto_official_row_count,
        crypto_only_row_count,
        skipped_missing_provenance: 0,
        skipped_missing_preflight: 0,
        skipped_research_only: 0,
        skipped_fixture: 0,
        skipped_summary_derived: 0,
        reason_codes: Vec::new(),
    }
}

fn references(
    official_ready_reference_count: usize,
    outcome_reference_count: usize,
    baseline_reference_count: usize,
    no_trade_counterfactual_count: usize,
    risk_denied_counterfactual_count: usize,
    diagnostic_only_reference_count: usize,
) -> OfficialReferenceReplicationArtifacts {
    OfficialReferenceReplicationArtifacts {
        report: OfficialReferenceReplicationReport {
            replication_id: "sufficiency".to_string(),
            generated_reference_pack: None,
            reference_pack_quality: None,
            outcome_reference_count,
            baseline_reference_count,
            no_trade_counterfactual_count,
            risk_denied_counterfactual_count,
            official_ready_reference_count,
            controlled_reference_count: 0,
            crypto_only_reference_count: 0,
            research_only_reference_count: 0,
            diagnostic_only_reference_count,
            replication_status: soma_zero::OfficialReferenceReplicationStatus::Unknown,
            reason_codes: Vec::new(),
        },
        bundle: None,
        linked_pack: None,
        closure_report: None,
    }
}

#[test]
fn sufficiency_reports_missing_official_rows_and_previous_controlled_status() {
    let previous_path = common::output_dir("sufficiency-previous").join("closure.txt");
    fs::write(
        &previous_path,
        "current_status=SufficientForCommitteeBenchmark\n",
    )
    .expect("write previous status");
    let config = soma_zero::OfficialEvidenceReplicationConfig {
        previous_sufficiency_closure_paths: vec![previous_path.display().to_string()],
        ..soma_zero::OfficialEvidenceReplicationConfig::default()
    };
    let report = OfficialSufficiencyReplicationBuilder::default().build(
        &config,
        &injection(Vec::new(), 0, 0, 0),
        None,
    );
    assert_eq!(
        report.previous_controlled_status,
        Some(soma_zero::CommitteeEvidenceSufficiencyStatus::SufficientForCommitteeBenchmark)
    );
    assert_eq!(
        report.final_status,
        OfficialSufficiencyReplicationStatus::MissingOfficialRows
    );
}

#[test]
fn sufficiency_reports_crypto_only_and_controlled_only_paths() {
    let mut crypto = row("sufficiency-crypto");
    crypto.market = soma_zero::ProviderMarket::Crypto;
    let crypto_report = OfficialSufficiencyReplicationBuilder::default().build(
        &soma_zero::OfficialEvidenceReplicationConfig::default(),
        &injection(vec![crypto], 1, 0, 1),
        Some(&references(1, 1, 1, 1, 1, 0)),
    );
    assert_eq!(
        crypto_report.final_status,
        OfficialSufficiencyReplicationStatus::CryptoOnlySufficiency
    );

    let mut controlled = row("sufficiency-controlled");
    controlled.provenance_summary = "controlled-local".to_string();
    let controlled_report = OfficialSufficiencyReplicationBuilder::default().build(
        &soma_zero::OfficialEvidenceReplicationConfig::default(),
        &injection(vec![controlled], 1, 1, 0),
        Some(&references(0, 1, 1, 1, 1, 0)),
    );
    assert_eq!(
        controlled_report.final_status,
        OfficialSufficiencyReplicationStatus::ControlledSufficiencyOnly
    );
}

#[test]
fn sufficiency_reports_missing_reference_components() {
    let config = soma_zero::OfficialEvidenceReplicationConfig::default();
    let injection = injection(vec![row("sufficiency-missing")], 1, 1, 0);
    let missing_outcomes = OfficialSufficiencyReplicationBuilder::default().build(
        &config,
        &injection,
        Some(&references(1, 0, 1, 1, 1, 0)),
    );
    assert_eq!(
        missing_outcomes.final_status,
        OfficialSufficiencyReplicationStatus::MissingOutcomeLinks
    );

    let missing_baselines = OfficialSufficiencyReplicationBuilder::default().build(
        &config,
        &injection,
        Some(&references(1, 1, 0, 1, 1, 0)),
    );
    assert_eq!(
        missing_baselines.final_status,
        OfficialSufficiencyReplicationStatus::MissingBaselineReferences
    );

    let missing_counterfactuals = OfficialSufficiencyReplicationBuilder::default().build(
        &config,
        &injection,
        Some(&references(1, 1, 1, 0, 1, 0)),
    );
    assert_eq!(
        missing_counterfactuals.final_status,
        OfficialSufficiencyReplicationStatus::MissingCounterfactuals
    );
}

#[test]
fn sufficiency_reports_summary_derived_and_pass_states() {
    let mut summary_row = row("sufficiency-summary");
    summary_row.materialization_level =
        soma_zero::CommitteeScenarioMaterializationLevel::BenchmarkSummary;
    let summary_report = OfficialSufficiencyReplicationBuilder::default().build(
        &soma_zero::OfficialEvidenceReplicationConfig::default(),
        &injection(vec![row("sufficiency-summary-a"), summary_row], 2, 2, 0),
        Some(&references(1, 1, 1, 1, 1, 0)),
    );
    assert_eq!(
        summary_report.final_status,
        OfficialSufficiencyReplicationStatus::TooMuchSummaryDerived
    );

    let pass_report = OfficialSufficiencyReplicationBuilder::default().build(
        &soma_zero::OfficialEvidenceReplicationConfig::default(),
        &injection(vec![row("sufficiency-pass")], 1, 1, 0),
        Some(&references(1, 1, 1, 1, 1, 0)),
    );
    assert!(pass_report.passed_for_official);
    assert_eq!(
        pass_report.final_status,
        OfficialSufficiencyReplicationStatus::OfficialSufficiencyPassed
    );
}

mod common;
#[path = "support/official_committee_support.rs"]
mod official_committee_support;

use soma_zero::{
    CommitteeCounterfactualBuildConfig, CommitteeCounterfactualBuilder,
    CommitteeEvidenceSufficiencyGateConfig, CommitteeEvidenceSufficiencyStatus,
    CommitteeOutcomeCoverageConfig, build_committee_outcome_coverage_report,
    build_committee_performance_evidence_matrix, evaluate_committee_evidence_sufficiency,
    load_local_candle_series_map,
};

fn baseline_inputs_named(
    name: &str,
) -> (
    soma_zero::CommitteeOutcomeCoverageReport,
    soma_zero::CommitteePerformanceEvidenceMatrix,
    soma_zero::CommitteeCounterfactualAuditReport,
) {
    let bundle = official_committee_support::build_controlled_benchmark_bundle(name, true);
    let candle_path =
        official_committee_support::write_candle_series(name, "AAPL", 1_700_000_000_000, 1.0);
    let series =
        load_local_candle_series_map(&[candle_path.display().to_string()]).expect("series");
    let records = bundle
        .outcome_linked_pack
        .linked_rows
        .iter()
        .flat_map(|row| {
            CommitteeCounterfactualBuilder::default().build_records(
                row,
                series.get("AAPL"),
                &CommitteeCounterfactualBuildConfig::default(),
            )
        })
        .collect::<Vec<_>>();
    let coverage = build_committee_outcome_coverage_report(
        &CommitteeOutcomeCoverageConfig::default(),
        &[bundle.official_scenario_pack.clone()],
        &[bundle.outcome_linked_pack.clone()],
        &records,
    );
    let matrix = build_committee_performance_evidence_matrix(
        name,
        &coverage,
        &[bundle.outcome_linked_pack.clone()],
        &[bundle.committee_benchmark_report.replay_report.clone()],
        &records,
        false,
    );
    let audit = soma_zero::build_committee_counterfactual_audit_report("sufficiency", records, &[]);
    (coverage, matrix, audit)
}

#[test]
fn sufficiency_gate_passes_controlled_official_bundle_and_warns_about_scope() {
    let (coverage, matrix, audit) = baseline_inputs_named("sufficiency-pass");
    let result = evaluate_committee_evidence_sufficiency(
        &CommitteeEvidenceSufficiencyGateConfig::default(),
        &coverage,
        Some(&audit),
        &matrix,
    );
    assert!(result.passed);
    assert_eq!(
        result.sufficiency_status,
        CommitteeEvidenceSufficiencyStatus::SufficientForCommitteeBenchmark
    );
    assert!(
        result
            .warnings
            .iter()
            .any(|warning| warning.contains("six-person"))
    );
}

#[test]
fn sufficiency_gate_fails_conservative_thresholds() {
    let (mut coverage, matrix, mut audit) = baseline_inputs_named("sufficiency-fail");
    coverage.official_rows = 0;
    let result = evaluate_committee_evidence_sufficiency(
        &CommitteeEvidenceSufficiencyGateConfig::default(),
        &coverage,
        Some(&audit),
        &matrix,
    );
    assert_eq!(
        result.sufficiency_status,
        CommitteeEvidenceSufficiencyStatus::InsufficientOfficialRows
    );

    coverage.official_rows = 3;
    coverage.outcome_linked_rows = 0;
    let result = evaluate_committee_evidence_sufficiency(
        &CommitteeEvidenceSufficiencyGateConfig::default(),
        &coverage,
        Some(&audit),
        &matrix,
    );
    assert_eq!(
        result.sufficiency_status,
        CommitteeEvidenceSufficiencyStatus::InsufficientOutcomeLinks
    );

    coverage.outcome_linked_rows = 3;
    coverage.baseline_linked_rows = 0;
    let result = evaluate_committee_evidence_sufficiency(
        &CommitteeEvidenceSufficiencyGateConfig::default(),
        &coverage,
        Some(&audit),
        &matrix,
    );
    assert_eq!(
        result.sufficiency_status,
        CommitteeEvidenceSufficiencyStatus::InsufficientBaselineReferences
    );

    coverage.baseline_linked_rows = 3;
    audit.no_trade_count = 0;
    let result = evaluate_committee_evidence_sufficiency(
        &CommitteeEvidenceSufficiencyGateConfig::default(),
        &coverage,
        Some(&audit),
        &matrix,
    );
    assert_eq!(
        result.sufficiency_status,
        CommitteeEvidenceSufficiencyStatus::InsufficientCounterfactuals
    );

    audit.no_trade_count = 3;
    coverage.summary_derived_rows = 3;
    coverage.row_level_rows = 0;
    let result = evaluate_committee_evidence_sufficiency(
        &CommitteeEvidenceSufficiencyGateConfig::default(),
        &coverage,
        Some(&audit),
        &matrix,
    );
    assert_eq!(
        result.sufficiency_status,
        CommitteeEvidenceSufficiencyStatus::TooMuchSummaryDerived
    );

    coverage.summary_derived_rows = 0;
    coverage.row_level_rows = 3;
    coverage.no_lookahead_violations = 1;
    let result = evaluate_committee_evidence_sufficiency(
        &CommitteeEvidenceSufficiencyGateConfig::default(),
        &coverage,
        Some(&audit),
        &matrix,
    );
    assert_eq!(
        result.sufficiency_status,
        CommitteeEvidenceSufficiencyStatus::NoLookaheadViolation
    );
}

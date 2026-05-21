mod common;
#[path = "support/official_committee_support.rs"]
mod official_committee_support;

use std::fs;

use soma_zero::{
    CommitteeReferencePackRunner, SufficiencyClosureConfig, SufficiencyClosureRunner,
    SufficiencyClosureStatus,
};

#[test]
fn sufficiency_closure_handles_missing_previous_and_computes_added_counts() {
    let config = official_committee_support::controlled_reference_pack_config("closure-official");
    let pack = CommitteeReferencePackRunner::default()
        .build_reference_pack(&config)
        .expect("pack");
    let previous_path = common::output_dir("closure-prev").join("coverage.txt");
    fs::write(
        &previous_path,
        [
            "coverage_id=prev",
            "total_rows=3",
            "official_rows=1",
            "outcome_linked_rows=0",
            "baseline_linked_rows=0",
            "no_trade_counterfactuals=0",
            "risk_denied_counterfactuals=0",
            "no_lookahead_violations=0",
        ]
        .join("\n"),
    )
    .expect("write previous coverage");
    let report = SufficiencyClosureRunner::default()
        .run_with_pack(
            &SufficiencyClosureConfig {
                closure_id: "closure-official".to_string(),
                previous_coverage_report_path: Some(previous_path.display().to_string()),
                generated_reference_pack_path: common::output_dir("closure-pack")
                    .join("generated_reference_pack.json")
                    .display()
                    .to_string(),
                output_root: common::output_dir("closure-official-out")
                    .display()
                    .to_string(),
                ..SufficiencyClosureConfig::default()
            },
            &pack,
        )
        .expect("closure");
    assert!(report.added_outcome_links > 0);
    assert!(report.added_baseline_references > 0);
    assert!(report.improvement_detected);
    assert_eq!(
        report.closure_status,
        SufficiencyClosureStatus::SufficiencyGatePassedForControlledEvidence
    );
}

#[test]
fn sufficiency_closure_labels_controlled_pass_distinctly_and_is_deterministic() {
    let mut row = official_committee_support::scenario_row(
        "closure-controlled",
        0,
        "AAPL",
        1_700_000_000_000,
    );
    row.source_kind = soma_zero::CommitteeScenarioSourceKind::Fixture;
    row.evidence_source_kind = soma_zero::EvidenceSourceKind::TestFixture;
    let scenario_set_path =
        official_committee_support::write_scenario_set("closure-controlled", vec![row]);
    let config = soma_zero::CommitteeReferencePackConfig {
        reference_pack_id: "closure-controlled".to_string(),
        scenario_set_paths: vec![scenario_set_path.display().to_string()],
        candle_series_paths: vec![
            official_committee_support::write_candle_series(
                "closure-controlled",
                "AAPL",
                1_700_000_000_000,
                1.0,
            )
            .display()
            .to_string(),
        ],
        allow_controlled_fixture_references: true,
        output_root: common::output_dir("closure-controlled-out")
            .display()
            .to_string(),
        ..soma_zero::CommitteeReferencePackConfig::default()
    };
    let pack = CommitteeReferencePackRunner::default()
        .build_reference_pack(&config)
        .expect("pack");
    let first = SufficiencyClosureRunner::default()
        .run_with_pack(&SufficiencyClosureConfig::default(), &pack)
        .expect("closure");
    let second = SufficiencyClosureRunner::default()
        .run_with_pack(&SufficiencyClosureConfig::default(), &pack)
        .expect("closure");
    assert_eq!(first, second);
    assert_eq!(
        first.closure_status,
        SufficiencyClosureStatus::SufficiencyGatePassedForControlledEvidence
    );
}

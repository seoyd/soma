#[path = "support/sprint69_support.rs"]
mod support;

use soma_zero::{
    CommitteeCounterfactualReferencePackStatus, CommitteeOutcomeReferencePackStatus,
    CommitteeScenarioPackStatus, SequenceCorePrototypeComparisonRunner,
    TrainingDataArtifactPopulationStatus, TrainingDataPopulatedIntegrityStatus,
};

#[test]
fn sprint80_committee_expansion_reports_expected_counts() {
    let config = support::sprint80_committee_config_from_example(
        "soma_committee_evidence_expand_v2.toml",
        "committee-ready",
    );
    let bundle = SequenceCorePrototypeComparisonRunner::default()
        .run_committee_evidence_expand_v2(&config)
        .expect("committee bundle");
    assert_eq!(
        bundle.committee_scenario_pack_v2.scenario_status,
        CommitteeScenarioPackStatus::ScenarioPackReadyWithWarnings
    );
    assert_eq!(bundle.committee_scenario_pack_v2.official_scenario_count, 1);
    assert_eq!(bundle.committee_scenario_pack_v2.research_only_count, 1);
    assert_eq!(bundle.committee_scenario_pack_v2.diagnostic_only_count, 1);
    assert_eq!(
        bundle.committee_outcome_reference_pack_v2.pack_status,
        CommitteeOutcomeReferencePackStatus::OutcomeReferencePackReady
    );
    assert_eq!(
        bundle
            .committee_no_trade_risk_denied_reference_pack
            .pack_status,
        CommitteeCounterfactualReferencePackStatus::CounterfactualReferencePackReady
    );
}

#[test]
fn sprint80_population_and_integrity_reports_are_specific() {
    let config = support::sprint80_population_config_from_example(
        "soma_training_artifact_populate.toml",
        "population-ready",
    );
    let runner = SequenceCorePrototypeComparisonRunner::default();
    let report = runner
        .run_training_artifact_populate(&config)
        .expect("populate");
    assert_eq!(
        report.population_status,
        TrainingDataArtifactPopulationStatus::ArtifactsPopulated
    );
    assert_eq!(report.registry_entries_added, 1);
    assert_eq!(report.dataset_versions_added, 1);
    assert!(report.lineage_edges_added > 0);
    assert_eq!(report.source_class_entries_added, 2);

    let integrity = runner
        .run_training_populated_integrity(&config)
        .expect("integrity");
    assert_eq!(
        integrity.integrity_status,
        TrainingDataPopulatedIntegrityStatus::PopulatedIntegrityReady
    );

    let bad_csv = support::write_support_text(
        "sprint80-population-invalid",
        "bad.csv",
        "sequence_id,confidence,expected_return,expected_drawdown,p_stop_loss,p_take_profit,p_time_expired,rank_score\n",
    );
    let mut warning_config = support::sprint80_population_config_from_example(
        "soma_training_artifact_populate.toml",
        "population-warning",
    );
    warning_config
        .prediction_csv_paths
        .push("/does/not/exist.csv".to_string());
    warning_config.prediction_csv_paths.push(bad_csv);
    let warning = runner
        .run_training_artifact_populate(&warning_config)
        .expect("population with warnings");
    assert_eq!(
        warning.population_status,
        TrainingDataArtifactPopulationStatus::ArtifactsPopulatedWithWarnings
    );
    assert!(warning.invalid_artifacts > 0);

    let missing_ref_csv = support::write_support_text(
        "sprint80-population-missing-ref",
        "ref.csv",
        "sequence_id,confidence,expected_return,expected_drawdown,p_stop_loss,p_take_profit,p_time_expired,rank_score\nseq-001,0.9,0.1,0.02,0.1,0.6,0.3,0.9\n",
    );
    let mut missing_ref = support::sprint80_population_config_from_example(
        "soma_training_artifact_populate.toml",
        "population-missing-ref",
    );
    missing_ref.prediction_csv_paths = vec![missing_ref_csv.clone()];
    runner
        .run_training_artifact_populate(&missing_ref)
        .expect("populate missing ref");
    std::fs::remove_file(missing_ref_csv).expect("remove referenced file");
    assert_eq!(
        runner
            .run_training_populated_integrity(&missing_ref)
            .expect("integrity missing ref")
            .integrity_status,
        TrainingDataPopulatedIntegrityStatus::MissingReferencedPaths
    );
}

mod common;
#[path = "support/sprint65_support.rs"]
mod sprint65_support;

use soma_zero::ExternalModelResearchOpsRunner;

#[test]
fn same_fixture_input_produces_same_research_ops_bundle() {
    let first = sprint65_support::research_ops_config_from_example(
        "soma_external_model_research_ops.toml",
        "research-ops-determinism-first",
    );
    let second = sprint65_support::research_ops_config_from_example(
        "soma_external_model_research_ops.toml",
        "research-ops-determinism-second",
    );
    let first_bundle = ExternalModelResearchOpsRunner::default()
        .run(&first)
        .expect("run first research ops bundle");
    let second_bundle = ExternalModelResearchOpsRunner::default()
        .run(&second)
        .expect("run second research ops bundle");
    assert_eq!(
        first_bundle.lifecycle_records,
        second_bundle.lifecycle_records
    );
    assert_eq!(
        first_bundle.external_model_review_queue,
        second_bundle.external_model_review_queue
    );
    assert_eq!(
        first_bundle.external_model_watchlist,
        second_bundle.external_model_watchlist
    );
    assert_eq!(
        first_bundle.model_comparability_matrix,
        second_bundle.model_comparability_matrix
    );
    assert_eq!(
        first_bundle.artifact_completeness_scores,
        second_bundle.artifact_completeness_scores
    );
    assert_eq!(
        first_bundle.model_evidence_risk_profiles,
        second_bundle.model_evidence_risk_profiles
    );
    assert_eq!(
        first_bundle.model_leaderboard_changelog,
        second_bundle.model_leaderboard_changelog
    );
    assert_eq!(
        first_bundle.external_model_research_ops_report,
        second_bundle.external_model_research_ops_report
    );
    assert_eq!(
        first_bundle.control_tower_model_ops_panel_summary,
        second_bundle.control_tower_model_ops_panel_summary
    );
    assert_eq!(first_bundle.final_summary, second_bundle.final_summary);
}

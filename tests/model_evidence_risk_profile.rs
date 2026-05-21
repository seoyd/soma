mod common;
#[path = "support/sprint65_support.rs"]
mod sprint65_support;

use soma_zero::{
    CalibrationDriftReport, CalibrationDriftStatus, ConservativeExternalLeaderboard,
    ExternalModelResearchOpsRunner, ModelEvidenceRecommendedAction, ModelEvidenceRiskLevel,
};

fn risk_profile_for<'a>(
    profiles: &'a [soma_zero::ModelEvidenceRiskProfile],
    model_id: &str,
    model_version: &str,
) -> &'a soma_zero::ModelEvidenceRiskProfile {
    profiles
        .iter()
        .find(|profile| profile.model_id == model_id && profile.model_version == model_version)
        .expect("risk profile")
}

#[test]
fn low_risk_profile_is_possible_with_clean_inputs() {
    let mut config = sprint65_support::research_ops_config_from_example(
        "soma_model_risk_profile.toml",
        "risk-profile-low",
    );
    config.owner_model_review_paths[0] = sprint65_support::write_support_json(
        "risk-profile-low",
        "owner_actions.json",
        &Vec::<serde_json::Value>::new(),
    );
    let mut drift: CalibrationDriftReport =
        sprint65_support::read_json(&config.calibration_drift_paths[0]);
    for record in &mut drift.records {
        if record.model_id == "ext-model-a" {
            record.calibration_status = CalibrationDriftStatus::Stable;
        }
    }
    drift.insufficient_history_count = 0;
    drift.stable_count = drift.records.len();
    config.calibration_drift_paths[0] =
        sprint65_support::write_support_json("risk-profile-low", "drift.json", &drift);

    let profiles = ExternalModelResearchOpsRunner::default()
        .run_risk_profile(&config)
        .expect("run low risk profile");
    let profile = risk_profile_for(&profiles, "ext-model-a", "1.1.0");
    assert_eq!(profile.evidence_risk_level, ModelEvidenceRiskLevel::Low);
    assert_eq!(
        profile.recommended_action,
        ModelEvidenceRecommendedAction::KeepResearchCandidate
    );
}

#[test]
fn low_coverage_and_poor_calibration_increase_risk() {
    let config = sprint65_support::research_ops_config_from_example(
        "soma_model_risk_profile.toml",
        "risk-profile-base",
    );
    let profiles = ExternalModelResearchOpsRunner::default()
        .run_risk_profile(&config)
        .expect("run base risk profile");
    let profile = risk_profile_for(&profiles, "ext-model-b", "1.0.0");
    assert_eq!(
        profile.evidence_risk_level,
        ModelEvidenceRiskLevel::Critical
    );
    assert_eq!(
        profile.recommended_action,
        ModelEvidenceRecommendedAction::RequestMorePredictions
    );
}

#[test]
fn poor_risk_behavior_and_unstable_ablation_increase_risk() {
    let mut config = sprint65_support::research_ops_config_from_example(
        "soma_model_risk_profile.toml",
        "risk-profile-risk-review",
    );
    let mut leaderboard: ConservativeExternalLeaderboard =
        sprint65_support::read_json(&config.conservative_leaderboard_paths[0]);
    if let Some(entry) = leaderboard
        .entries
        .iter_mut()
        .find(|entry| entry.model_id == "ext-model-a" && entry.model_version == "1.1.0")
    {
        entry.risk_adjusted_score = Some(-0.1);
        entry.ablation_status = Some("Unstable".to_string());
    }
    config.conservative_leaderboard_paths[0] = sprint65_support::write_support_json(
        "risk-profile-risk-review",
        "leaderboard.json",
        &leaderboard,
    );
    config.owner_model_review_paths[0] = sprint65_support::write_support_json(
        "risk-profile-risk-review",
        "owner_actions.json",
        &Vec::<serde_json::Value>::new(),
    );

    let profiles = ExternalModelResearchOpsRunner::default()
        .run_risk_profile(&config)
        .expect("run risk review profile");
    let profile = risk_profile_for(&profiles, "ext-model-a", "1.1.0");
    assert_eq!(profile.evidence_risk_level, ModelEvidenceRiskLevel::High);
    assert_eq!(
        profile.recommended_action,
        ModelEvidenceRecommendedAction::RequestRiskReview
    );
}

#[test]
fn risk_profile_is_deterministic() {
    let first = sprint65_support::research_ops_config_from_example(
        "soma_model_risk_profile.toml",
        "risk-profile-determinism-first",
    );
    let second = sprint65_support::research_ops_config_from_example(
        "soma_model_risk_profile.toml",
        "risk-profile-determinism-second",
    );
    let first_profiles = ExternalModelResearchOpsRunner::default()
        .run_risk_profile(&first)
        .expect("run first risk profile");
    let second_profiles = ExternalModelResearchOpsRunner::default()
        .run_risk_profile(&second)
        .expect("run second risk profile");
    assert_eq!(first_profiles, second_profiles);
}

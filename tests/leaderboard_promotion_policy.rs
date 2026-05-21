mod common;
#[path = "support/sprint64_support.rs"]
mod sprint64_support;

use soma_zero::{ExternalArtifactRegistryRunner, LeaderboardPromotionPolicyStatus};

#[test]
fn default_policy_is_ready_and_stays_research_only() {
    let config = sprint64_support::registry_config_from_example(
        "soma_external_artifact_registry.toml",
        "policy-default",
    );
    let report = ExternalArtifactRegistryRunner::default()
        .run(&config)
        .expect("run registry bundle")
        .leaderboard_promotion_policy_report;
    assert_eq!(
        report.policy_status,
        LeaderboardPromotionPolicyStatus::PolicyReady
    );
    assert!(report.research_candidate_count >= 1);
    assert!(
        report
            .model_results
            .iter()
            .all(|result| !result.to_ascii_lowercase().contains("live"))
    );
}

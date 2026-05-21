mod support;

use support::sprint98_support::run_sprint98;

#[test]
fn learning_policies_allow_offline_study_and_block_training_or_broker_access() {
    let first = run_sprint98(
        "soma_sprint98_committee_owned_core.toml",
        "committee-member-learning-policy",
    );
    let second = run_sprint98(
        "soma_sprint98_committee_owned_core.toml",
        "committee-member-learning-policy-second",
    );
    for policy in &first.ai_committee_member_learning_policies {
        assert!(policy.can_read_historical_data);
        assert!(policy.can_read_official_data);
        assert!(policy.can_read_research_data);
        assert!(policy.can_generate_study_notes);
        assert!(policy.can_update_member_scorecard);
        assert!(!policy.can_update_model_weights);
        assert!(!policy.can_train_model);
        assert!(!policy.can_use_live_data_for_training);
        assert!(!policy.can_access_broker_account);
    }
    assert_eq!(
        first.ai_committee_member_learning_policies,
        second.ai_committee_member_learning_policies
    );
}

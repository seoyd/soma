mod support;

use soma_zero::{
    FamilyReductionAction, FamilyReductionPlanStatus, Sprint88SevenBlockerRecoveryRunner,
    TestBinaryConsolidationSemanticRisk,
};
use support::sprint69_support as sprint;

#[test]
fn family_reduction_plans_preserve_actions_and_risk() {
    let bundle = sprint::run_sprint88_bundle(
        "soma_sprint88_seven_blocker_recover.toml",
        "family-reduction-bundle",
    );
    let committee = bundle
        .family_reduction_plans
        .iter()
        .find(|plan| format!("{:?}", plan.family) == "CommitteeCliSafety")
        .expect("committee plan");
    assert_eq!(
        committee.plan_status,
        FamilyReductionPlanStatus::NeedManualReview
    );
    assert_eq!(
        committee.semantic_risk,
        TestBinaryConsolidationSemanticRisk::High
    );
    assert!(
        committee
            .actions
            .contains(&FamilyReductionAction::KeepSeparateForIsolation)
    );
    let external = bundle
        .family_reduction_plans
        .iter()
        .find(|plan| format!("{:?}", plan.family) == "ExternalPrediction")
        .expect("external plan");
    assert!(
        external
            .actions
            .contains(&FamilyReductionAction::CollapseFeatureVariant)
    );
}

#[test]
fn family_reduction_reports_are_deterministic() {
    let config =
        sprint::sprint88_config_from_example("soma_sprint88_seven_blocker_recover.toml", "reduce");
    let first = Sprint88SevenBlockerRecoveryRunner::default()
        .run(&config)
        .expect("first");
    let second = Sprint88SevenBlockerRecoveryRunner::default()
        .run(&config)
        .expect("second");
    assert_eq!(
        first.family_reduction_reports,
        second.family_reduction_reports
    );
}

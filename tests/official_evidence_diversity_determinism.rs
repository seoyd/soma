#[path = "support/sprint48_support.rs"]
mod support;

use soma_zero::{
    BalancedOutcomeCoverageRunner, BarrierProfileRegistryBuilder,
    DiversityAwareSufficiencyV2Runner, OfficialDiversityRowSelector,
    OfficialEvidenceDiversityGapRunner, OfficialEvidenceDiversitySweepRunner,
    OutcomeDiversityAuditRunner,
};

#[test]
fn sprint48_example_inputs_produce_stable_outputs() {
    let gap_config = support::diversity_gap_config("det-gap");
    let selector_config = support::row_selector_config("det-selector");
    let audit_config = support::outcome_audit_config("det-audit");
    let coverage_config = support::balanced_coverage_config("det-coverage");
    let sufficiency_config = support::sufficiency_config("det-sufficiency");
    let sweep_config = support::sweep_multi_config("det-sweep");

    let gap_a = OfficialEvidenceDiversityGapRunner::default()
        .run(&gap_config)
        .expect("gap a");
    let gap_b = OfficialEvidenceDiversityGapRunner::default()
        .run(&gap_config)
        .expect("gap b");
    assert_eq!(gap_a.to_text(), gap_b.to_text());

    let selector_a = OfficialDiversityRowSelector::default()
        .run(&selector_config)
        .expect("selector a");
    let selector_b = OfficialDiversityRowSelector::default()
        .run(&selector_config)
        .expect("selector b");
    assert_eq!(selector_a.to_text(), selector_b.to_text());

    let audit_a = OutcomeDiversityAuditRunner::default()
        .run(&audit_config)
        .expect("audit a");
    let audit_b = OutcomeDiversityAuditRunner::default()
        .run(&audit_config)
        .expect("audit b");
    assert_eq!(audit_a.to_text(), audit_b.to_text());

    let coverage_a = BalancedOutcomeCoverageRunner::default()
        .run(&coverage_config)
        .expect("coverage a");
    let coverage_b = BalancedOutcomeCoverageRunner::default()
        .run(&coverage_config)
        .expect("coverage b");
    assert_eq!(coverage_a.to_text(), coverage_b.to_text());

    let sufficiency_a = DiversityAwareSufficiencyV2Runner::default()
        .run(&sufficiency_config)
        .expect("sufficiency a");
    let sufficiency_b = DiversityAwareSufficiencyV2Runner::default()
        .run(&sufficiency_config)
        .expect("sufficiency b");
    assert_eq!(sufficiency_a.to_text(), sufficiency_b.to_text());

    let sweep_a = OfficialEvidenceDiversitySweepRunner::default()
        .run(&sweep_config)
        .expect("sweep a");
    let sweep_b = OfficialEvidenceDiversitySweepRunner::default()
        .run(&sweep_config)
        .expect("sweep b");
    assert_eq!(sweep_a.final_summary, sweep_b.final_summary);
    assert_eq!(
        sweep_a.diversity_sweep_report.to_text(),
        sweep_b.diversity_sweep_report.to_text()
    );
}

#[test]
fn barrier_profile_selection_is_not_post_hoc_outcome_forced() {
    let registry = BarrierProfileRegistryBuilder::default()
        .build(&support::barrier_profiles_primary("det-barrier-registry"))
        .expect("registry");
    let selected = registry.official_profile(None).expect("official profile");

    assert_eq!(selected.profile_id, "primary-preregistered");
    assert!(selected.registered_before_outcome_eval);
    assert!(selected.official_sufficiency_eligible());
}

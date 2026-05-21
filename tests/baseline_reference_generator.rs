mod common;
#[path = "support/official_committee_support.rs"]
mod official_committee_support;

use soma_zero::{
    BaselineReferenceGenerator, BaselineReferencePolicy, BaselineReferenceSource,
    CommitteeBaselineAction,
};

#[test]
fn baseline_generator_prefers_existing_artifacts_and_is_deterministic() {
    let row = official_committee_support::scenario_row("baseline", 0, "AAPL", 1_700_000_000_000);
    let artifacts =
        BaselineReferenceGenerator::load_existing(&[official_committee_support::write_baselines(
            "baseline",
        )
        .display()
        .to_string()])
        .expect("artifacts");
    let existing = BaselineReferenceGenerator::find_existing(&row, &artifacts, 24);
    let first = BaselineReferenceGenerator::default().generate(
        &row,
        existing,
        &BaselineReferencePolicy {
            source: BaselineReferenceSource::ExistingArtifact,
            ..BaselineReferencePolicy::default()
        },
    );
    let second = BaselineReferenceGenerator::default().generate(
        &row,
        existing,
        &BaselineReferencePolicy {
            source: BaselineReferenceSource::ExistingArtifact,
            ..BaselineReferencePolicy::default()
        },
    );
    assert_eq!(first, second);
    assert_eq!(
        first.reference.baseline_action,
        CommitteeBaselineAction::Approve
    );
    assert!(!first.diagnostic_only);
}

#[test]
fn baseline_generator_has_no_trade_fallback_and_reason_coded_approximation() {
    let row =
        official_committee_support::scenario_row("baseline-fallback", 0, "AAPL", 1_700_000_000_010);
    let fallback = BaselineReferenceGenerator::default().generate(
        &row,
        None,
        &BaselineReferencePolicy::default(),
    );
    assert_eq!(
        fallback.reference.baseline_action,
        CommitteeBaselineAction::NoTrade
    );

    let approx = BaselineReferenceGenerator::default().generate(
        &row,
        None,
        &BaselineReferencePolicy {
            source: BaselineReferenceSource::DeterministicBaselineSignalApprox,
            allow_approximation: false,
            ..BaselineReferencePolicy::default()
        },
    );
    assert!(approx.diagnostic_only);
    assert!(
        approx
            .reason_codes
            .iter()
            .any(|code| format!("{:?}", code) == "EvidenceEstimateBuilt")
    );
}

#[path = "support/sprint47_support.rs"]
mod sprint47_support;

use soma_zero::{OfficialEvidenceSufficiencyV2Runner, OfficialEvidenceSufficiencyV2Status};

#[test]
fn sufficiency_remains_conservative_for_single_row_and_validates_plumbing_for_multi_row() {
    let single = OfficialEvidenceSufficiencyV2Runner::default()
        .run(&sprint47_support::example_sufficiency(
            "single-sufficiency",
            "examples/soma_official_evidence_sufficiency_v2_single_row.toml",
        ))
        .expect("single sufficiency");
    assert!(single.still_insufficient_for_usefulness_claims);
    assert_eq!(
        single.sufficiency_status,
        OfficialEvidenceSufficiencyV2Status::InsufficientRows
    );

    let first = OfficialEvidenceSufficiencyV2Runner::default()
        .run(&sprint47_support::example_sufficiency(
            "multi-sufficiency-a",
            "examples/soma_official_evidence_sufficiency_v2_multi_row.toml",
        ))
        .expect("multi sufficiency");
    let second = OfficialEvidenceSufficiencyV2Runner::default()
        .run(&sprint47_support::example_sufficiency(
            "multi-sufficiency-b",
            "examples/soma_official_evidence_sufficiency_v2_multi_row.toml",
        ))
        .expect("multi sufficiency second");
    assert!(first.passed_plumbing_validation);
    assert!(!first.passed_committee_benchmark_research);
    assert_eq!(
        first.sufficiency_status,
        OfficialEvidenceSufficiencyV2Status::PlumbingValidated
    );
    assert_eq!(first.to_text(), second.to_text());
}

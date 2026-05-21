#[path = "support/sprint69_support.rs"]
mod support;

use soma_zero::RetirementEvidenceCompletionStatus;

#[test]
fn retirement_evidence_completion_matches_fixture_and_stays_conservative() {
    let bundle = support::run_briefing(
        "soma_retirement_evidence_completion.toml",
        "retirement-evidence-completion",
    );
    let expected = support::read_json::<soma_zero::RetirementEvidenceCompletionReport>(
        support::example_path("sprint71_data/retirement_evidence_ext_model_a_1_0_0.json"),
    );
    assert_eq!(bundle.retirement_evidence_completion_report, expected);
    assert!(
        !bundle
            .retirement_evidence_completion_report
            .supports_retirement
    );
    assert!(
        bundle
            .retirement_evidence_completion_report
            .supports_diagnostic_downgrade
    );
    assert_eq!(
        bundle
            .retirement_evidence_completion_report
            .completion_status,
        RetirementEvidenceCompletionStatus::NeedsMoreRegressionEvidence
    );
    assert!(
        !bundle
            .retirement_evidence_completion_report
            .conservative_action
            .to_ascii_lowercase()
            .contains("delete")
    );
}

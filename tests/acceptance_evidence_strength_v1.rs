mod support;

use soma_zero::AcceptanceEvidenceStrengthReportV1;
use support::sprint111_support::{read_fixture, run_sprint111};

#[test]
fn acceptance_evidence_strength_v1_matches_fixture() {
    let bundle = run_sprint111(
        "soma_acceptance_evidence_strength_v1.toml",
        "acceptance-evidence-strength-v1",
    );
    let expected: AcceptanceEvidenceStrengthReportV1 =
        read_fixture("sprint111_data/acceptance_evidence_strength_expected.json");
    assert_eq!(bundle.acceptance_evidence_strength_report_v1, expected);
    assert_eq!(
        bundle
            .acceptance_evidence_strength_report_v1
            .focused_evidence_strength,
        "SupportingOnly"
    );
    assert_eq!(
        bundle
            .acceptance_evidence_strength_report_v1
            .full_workspace_evidence_strength,
        "Insufficient"
    );
}

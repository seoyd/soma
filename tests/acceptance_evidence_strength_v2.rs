mod support;

use soma_zero::AcceptanceEvidenceStrengthReportV2;
use support::sprint112_support::{read_fixture, run_sprint112};

#[test]
fn acceptance_evidence_strength_v2_requires_full_pass() {
    let bundle = run_sprint112(
        "soma_acceptance_evidence_strength_v2.toml",
        "acceptance-evidence-v2",
    );
    let expected: AcceptanceEvidenceStrengthReportV2 =
        read_fixture("sprint112_data/acceptance_evidence_strength_v2_expected.json");
    assert_eq!(bundle.acceptance_evidence_strength_report_v2, expected);
    assert_eq!(
        bundle
            .acceptance_evidence_strength_report_v2
            .focused_evidence_strength,
        "SupportingOnly"
    );
    assert_eq!(
        bundle
            .acceptance_evidence_strength_report_v2
            .nextest_evidence_strength,
        "Insufficient"
    );
    assert_eq!(
        bundle
            .acceptance_evidence_strength_report_v2
            .full_workspace_evidence_strength,
        "Insufficient"
    );
    assert_eq!(
        bundle.acceptance_truth_gate_v13.truth_status,
        "AcceptanceTruthReadyWithWarnings"
    );
}

mod support;

use soma_zero::EvidenceBlurRiskReportV1;
use support::sprint115_support::{read_fixture, run_sprint115};

#[test]
fn evidence_blur_risk_gate_v1_blocks_when_blur_is_high() {
    let bundle = run_sprint115(
        "soma_evidence_blur_risk_gate_v1.toml",
        "evidence-blur-risk-gate-v1",
    );
    let expected: EvidenceBlurRiskReportV1 =
        read_fixture("sprint115_data/evidence_blur_risk_expected.json");
    assert_eq!(bundle.evidence_blur_risk_report_v1, expected);
    assert_eq!(
        bundle.evidence_blur_risk_gate_v1.gate_status,
        "EvidenceBlurRiskTooHigh"
    );
}

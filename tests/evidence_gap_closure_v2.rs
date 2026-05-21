#[path = "support/sprint69_support.rs"]
mod support;

use soma_zero::EvidenceGapClosureV2Status;

#[test]
fn evidence_gap_closure_v2_reduces_gaps_but_keeps_remaining_data_request_visible() {
    let bundle = support::run_offline_attachment(
        "soma_offline_evidence_attach.toml",
        "evidence-gap-closure-v2",
    );
    let report = bundle.evidence_gap_closure_v2_report;

    assert_eq!(report.gaps_before, 6);
    assert_eq!(report.gaps_after, 1);
    assert_eq!(report.remaining_gaps, vec!["ext-model-b:1.0.0".to_string()]);
    assert_eq!(
        report.closure_status,
        EvidenceGapClosureV2Status::StillNeedsData
    );
}

mod support;

use soma_zero::DebateEvidenceSufficiencyStatus;
use support::sprint99_support::run_sprint99;

#[test]
fn debate_evidence_sufficiency_is_paper_safe() {
    let bundle = run_sprint99(
        "soma_debate_evidence_sufficiency.toml",
        "debate-evidence-sufficiency",
    );
    let report = bundle.debate_evidence_sufficiency_report;
    assert_eq!(
        report.evidence_status,
        DebateEvidenceSufficiencyStatus::EvidenceSufficientForPaperDebate
    );
    assert_eq!(report.evidence_ref_count, 10);
    assert!(report.source_boundary_ok);
    assert!(report.no_lookahead_ok);
    assert!(report.missing_evidence_kinds.is_empty());
}

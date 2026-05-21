#[path = "support/sprint69_support.rs"]
mod support;

use soma_zero::EvidenceGapKind;

#[test]
fn evidence_gap_closure_checklist_contains_expected_gap_kinds_and_counts() {
    let bundle = support::run_briefing(
        "soma_operator_briefing.toml",
        "evidence-gap-closure-checklist",
    );
    let kinds = bundle
        .evidence_gap_closure_checklist
        .gaps
        .iter()
        .map(|item| item.gap_kind)
        .collect::<Vec<_>>();
    assert!(kinds.contains(&EvidenceGapKind::MissingPredictionHistory));
    assert!(kinds.contains(&EvidenceGapKind::MissingLeaderboardEvidence));
    assert!(kinds.contains(&EvidenceGapKind::MissingRetirementEvidence));
    assert!(kinds.contains(&EvidenceGapKind::MissingOwnerReason));
    assert!(bundle.evidence_gap_closure_checklist.closeable_now_count > 0);
    assert!(bundle.evidence_gap_closure_checklist.requires_data_count > 0);
}

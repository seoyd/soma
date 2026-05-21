mod support;

use soma_zero::LowerConfidenceHardeningAction;
use support::sprint102_support::run_sprint102;

#[test]
fn lower_confidence_evidence_hardening_targets_warning_backed_candidates() {
    let bundle = run_sprint102(
        "soma_lower_confidence_evidence_hardening.toml",
        "sprint102-lower-confidence",
    );
    let plan = &bundle.lower_confidence_evidence_hardening_plan;
    assert_eq!(
        plan.target_candidates,
        vec!["arthur-hayes", "larry-williams", "wonyotti"]
    );
    let report = &bundle.lower_confidence_evidence_hardening_report;
    assert_eq!(report.candidate_count, 3);
    assert_eq!(report.still_warning_candidates.len(), 3);
    assert!(!report.downweighted_items.is_empty());
    assert!(
        plan.recommended_actions["wonyotti"]
            .contains(&LowerConfidenceHardeningAction::DownWeightCommunityAnecdote)
    );
}

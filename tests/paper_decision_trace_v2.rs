mod support;

use support::sprint102_support::run_sprint102;

#[test]
fn paper_decision_trace_and_coverage_stay_complete() {
    let bundle = run_sprint102("soma_paper_decision_trace_v2.toml", "sprint102-trace");
    let trace = &bundle.paper_decision_trace_v2;
    assert!(!trace.proposal_run_ref.is_empty());
    assert!(!trace.debate_session_ref.is_empty());
    assert!(!trace.risk_governor_handoff_ref.is_empty());
    assert!(!trace.broker_execution_allowed);
    assert!(!trace.live_execution_allowed);
    assert!(
        bundle
            .regime_routed_committee_dry_run_report
            .routed_to_short_term_count
            > 0
    );
    assert!(
        bundle
            .regime_routed_committee_dry_run_report
            .routed_to_long_term_count
            > 0
    );
    assert!(
        bundle
            .regime_routed_committee_dry_run_report
            .routed_to_crypto_count
            > 0
    );
    assert!(
        bundle
            .regime_routed_committee_dry_run_report
            .routed_to_common_risk_count
            > 0
    );
    assert!(
        bundle
            .multi_expert_rotation_coverage_report
            .total_members_selected
            > 0
    );
    assert!(
        !bundle
            .paper_roster_expansion_usage_report
            .live_expansion_allowed
    );
    assert_eq!(
        bundle
            .eighteen_archetype_activation_safety_report
            .live_activation_attempt_count,
        0
    );
}

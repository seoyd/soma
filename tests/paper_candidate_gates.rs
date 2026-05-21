mod support;

use support::sprint104_support::run_sprint104;

#[test]
fn paper_candidate_gates_preserve_paper_only_semantics() {
    let bundle = run_sprint104(
        "soma_sprint104_dual_agent_paper_lifecycle.toml",
        "paper_candidate_gates",
    );
    assert!(!bundle.paper_candidate_promotion_gate.live_execution_allowed);
    assert!(
        bundle
            .paper_candidate_promotion_gate
            .gate_status
            .contains("PaperPromotion")
            || bundle
                .paper_candidate_promotion_gate
                .gate_status
                .contains("NeedMoreEvidence")
    );
    assert!(
        bundle
            .paper_candidate_rejection_gate
            .gate_status
            .starts_with("PaperRejectionReady")
    );
    assert!(
        bundle
            .paper_candidate_watchlist_gate
            .watchlist_status
            .starts_with("Watchlist")
    );
    assert!(
        bundle
            .paper_candidate_no_trade_gate
            .gate_status
            .starts_with("NoTradeGate")
    );
    assert!(
        bundle
            .paper_candidate_risk_denied_gate
            .gate_status
            .starts_with("RiskDeniedGate")
    );
}

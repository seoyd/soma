use soma_zero::{EvidenceSourceKind, EvidenceUse};

#[test]
fn yfinance_source_kind_is_research_only() {
    assert_eq!(
        EvidenceSourceKind::YFinanceResearch.readiness_eligible(),
        false
    );
    assert!(EvidenceSourceKind::YFinanceResearch.supports(EvidenceUse::BacktestResearch));
    assert!(!EvidenceSourceKind::YFinanceResearch.supports(EvidenceUse::ReadinessEvidence));
    assert!(EvidenceSourceKind::YFinanceResearch.supports(EvidenceUse::DisallowedForReadiness));
}

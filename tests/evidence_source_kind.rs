use soma_zero::{EvidenceSourceKind, EvidenceUse};

#[test]
fn synthetic_like_sources_are_not_readiness_eligible() {
    for kind in [
        EvidenceSourceKind::SyntheticFixture,
        EvidenceSourceKind::TestFixture,
        EvidenceSourceKind::GeneratedSynthetic,
        EvidenceSourceKind::Unknown,
    ] {
        assert!(!kind.readiness_eligible());
        assert!(!kind.supports(EvidenceUse::ReadinessEvidence));
    }
}

#[test]
fn real_local_can_be_readiness_eligible() {
    assert!(EvidenceSourceKind::RealLocal.readiness_eligible());
    assert!(EvidenceSourceKind::RealLocal.supports(EvidenceUse::RealDataEvidence));
    assert!(EvidenceSourceKind::RealLocal.supports(EvidenceUse::ReadinessEvidence));
}

#[test]
fn official_api_collected_can_be_readiness_eligible() {
    assert!(EvidenceSourceKind::OfficialApiCollected.readiness_eligible());
    assert!(EvidenceSourceKind::OfficialApiCollected.supports(EvidenceUse::RealDataEvidence));
    assert!(EvidenceSourceKind::OfficialApiCollected.supports(EvidenceUse::ReadinessEvidence));
}

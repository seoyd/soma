mod support;

use soma_zero::{InvestorStyleArchetype, InvestorStyleArchetypeKind, InvestorStyleStatus};
use support::shared_fixture_harness::load_json_fixture;
use support::sprint69_support::example_path;
use support::sprint98_support::run_sprint98;

#[test]
fn investor_style_registry_covers_required_safe_archetypes() {
    let bundle = run_sprint98(
        "soma_sprint98_committee_owned_core.toml",
        "investor-style-archetypes",
    );
    let fixture: Vec<InvestorStyleArchetype> =
        load_json_fixture(example_path("sprint98_data/investor_style_archetypes.json"));
    let kinds = bundle
        .investor_style_archetype_registry
        .styles
        .iter()
        .map(|style| style.archetype_kind)
        .collect::<Vec<_>>();
    for required in [
        InvestorStyleArchetypeKind::TrendFollower,
        InvestorStyleArchetypeKind::RiskFirstDefensive,
        InvestorStyleArchetypeKind::RegimeCycle,
        InvestorStyleArchetypeKind::ValueDiscipline,
        InvestorStyleArchetypeKind::MacroReflexive,
        InvestorStyleArchetypeKind::CounterfactualHistorian,
    ] {
        assert!(kinds.contains(&required), "missing {required:?}");
    }
    let unsafe_style = InvestorStyleArchetype {
        archetype_id: "unsafe".to_string(),
        archetype_kind: InvestorStyleArchetypeKind::ValueDiscipline,
        public_philosophy_inspiration: "exact reproduction of a private method".to_string(),
        decision_biases: vec![],
        preferred_evidence: vec!["official data".to_string()],
        risk_blindspots: vec!["none".to_string()],
        preferred_time_horizon: "long-term".to_string(),
        prohibited_claims: vec!["private method".to_string()],
        style_status: InvestorStyleStatus::StyleReady,
        reason_codes: vec![],
    }
    .validated();
    assert_eq!(
        unsafe_style.style_status,
        InvestorStyleStatus::UnsafeImpersonationBlocked
    );
    assert!(
        bundle
            .investor_style_archetype_registry
            .styles
            .iter()
            .all(|style| style
                .public_philosophy_inspiration
                .contains("public philosophy-inspired"))
    );
    assert!(
        bundle
            .investor_style_archetype_registry
            .styles
            .iter()
            .all(|style| !style.risk_blindspots.is_empty())
    );
    assert!(
        bundle
            .investor_style_archetype_registry
            .styles
            .iter()
            .all(|style| !style.preferred_evidence.is_empty())
    );
    assert_eq!(fixture, bundle.investor_style_archetype_registry.styles);
}

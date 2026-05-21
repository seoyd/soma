#[path = "support/sprint48_support.rs"]
mod support;

use soma_zero::{
    BarrierProfile, BarrierProfileIntendedUse, BarrierProfileKind, BarrierProfileRegistryBuilder,
};

#[test]
fn primary_preregistered_profile_is_accepted() {
    let config = support::barrier_profiles_primary("barrier-primary-accepted");
    let registry = BarrierProfileRegistryBuilder::default()
        .build(&config)
        .expect("build registry");

    assert!(registry.has_primary_profile());
    assert_eq!(registry.primary_profiles.len(), 1);
    assert_eq!(registry.official_sufficiency_eligible_profiles.len(), 2);
    assert_eq!(
        registry
            .official_profile(None)
            .map(|profile| profile.profile_id.as_str()),
        Some("primary-preregistered")
    );
}

#[test]
fn missing_primary_profile_fails_official_sufficiency_use() {
    let mut config = support::barrier_profiles_diagnostic("barrier-missing-primary");
    config.require_primary_profile = true;

    let error = BarrierProfileRegistryBuilder::default()
        .build(&config)
        .expect_err("missing primary should fail");
    assert!(error.contains("requires a primary preregistered profile"));
}

#[test]
fn diagnostic_profile_is_diagnostic_only() {
    let config = support::barrier_profiles_primary("barrier-diagnostic-only");
    let registry = BarrierProfileRegistryBuilder::default()
        .build(&config)
        .expect("build registry");
    let diagnostic = registry
        .diagnostic_profiles
        .iter()
        .find(|profile| profile.profile_id == "diagnostic-grid")
        .expect("diagnostic profile");

    assert!(diagnostic.diagnostic_only());
    assert!(!diagnostic.official_sufficiency_eligible());
}

#[test]
fn exploratory_profile_is_exploratory_only() {
    let config = support::barrier_profiles_diagnostic("barrier-exploratory-only");
    let registry = BarrierProfileRegistryBuilder::default()
        .build(&config)
        .expect("build registry");
    let exploratory = registry
        .exploratory_profiles
        .iter()
        .find(|profile| profile.profile_id == "exploratory-after-the-fact")
        .expect("exploratory profile");

    assert!(exploratory.diagnostic_only());
    assert!(!exploratory.official_sufficiency_eligible());
}

#[test]
fn registered_after_outcome_eval_cannot_satisfy_official_sufficiency() {
    let mut config = support::barrier_profiles_primary("barrier-after-outcome");
    config.profiles.push(BarrierProfile {
        profile_id: "secondary-after-outcome".to_string(),
        profile_kind: BarrierProfileKind::SecondaryPreregistered,
        horizon_bars: 7,
        take_profit_pct: 0.03,
        stop_loss_pct: 0.015,
        cost_bps: 5.0,
        slippage_bps: 2.0,
        tie_break_policy: Default::default(),
        intended_use: BarrierProfileIntendedUse::OfficialSufficiency,
        registered_before_outcome_eval: false,
        reason_codes: vec![],
    });

    let registry = BarrierProfileRegistryBuilder::default()
        .build(&config)
        .expect("build registry");
    assert!(
        registry
            .official_sufficiency_eligible_profiles
            .iter()
            .all(|profile| profile.profile_id != "secondary-after-outcome")
    );
    assert!(
        registry
            .warnings
            .iter()
            .any(|warning| warning.contains("excluded from official sufficiency"))
    );
}

#[test]
fn remote_profile_path_rejected() {
    let mut config = support::barrier_profiles_primary("barrier-remote-path");
    config.output_root = "https://example.com/out".to_string();

    let error = config
        .validate()
        .expect_err("remote output root should fail");
    assert!(error.contains("must be local"));
}

#[test]
fn registry_is_deterministic() {
    let config = support::barrier_profiles_primary("barrier-deterministic");

    let first = BarrierProfileRegistryBuilder::default()
        .build(&config)
        .expect("first build");
    let second = BarrierProfileRegistryBuilder::default()
        .build(&config)
        .expect("second build");

    assert_eq!(first, second);
    assert_eq!(first.to_text(), second.to_text());
    assert_eq!(first.fingerprint(), second.fingerprint());
}

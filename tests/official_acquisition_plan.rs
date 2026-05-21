use std::path::PathBuf;

use soma_zero::{
    OfficialEvidenceAcquisitionPlan, ProviderKind, ReasonCode,
    build_evidence_acquisition_storage_check,
};

fn example_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join(name)
}

#[test]
fn acquisition_plan_has_safe_defaults() {
    let plan = OfficialEvidenceAcquisitionPlan::default();

    assert!(plan.run_upbit_if_public_available);
    assert_eq!(plan.max_symbols, 3);
    assert!(!plan.allow_full_history);
    assert!(!plan.allow_all_symbols);
}

#[test]
fn acquisition_plan_rejects_remote_paths() {
    let plan = OfficialEvidenceAcquisitionPlan {
        output_root: "https://example.com/out".to_string(),
        ..OfficialEvidenceAcquisitionPlan::default()
    };

    assert!(
        plan.validate_local_paths()
            .contains(&ReasonCode::RemotePathRejected)
    );
}

#[test]
fn acquisition_scope_rules_are_blocked_by_storage_check() {
    let check = build_evidence_acquisition_storage_check(&OfficialEvidenceAcquisitionPlan {
        allow_all_symbols: true,
        allow_full_history: true,
        ..OfficialEvidenceAcquisitionPlan::default()
    });

    assert!(!check.budget_ok);
}

#[test]
fn sprint26_acquisition_examples_parse() {
    for path in [
        example_path("soma_official_evidence_acquisition.toml"),
        example_path("soma_official_evidence_acquisition_crypto_only.toml"),
        example_path("soma_official_evidence_acquisition_multi_venue.toml"),
    ] {
        let plan = OfficialEvidenceAcquisitionPlan::from_toml_path(&path).expect("parse example");
        assert!(
            plan.run_upbit_if_public_available
                || plan.run_krx_if_auth_ready
                || plan.run_alpha_if_auth_ready
        );
    }
}

#[test]
fn plan_surface_stays_research_only() {
    let toml = OfficialEvidenceAcquisitionPlan::default()
        .to_toml_string()
        .expect("serialize");

    assert!(!toml.contains("broker"));
    assert!(!toml.contains("account"));
    assert!(!toml.contains("llm"));
    assert!(!toml.contains(&format!("{:?}", ProviderKind::Unknown)));
}

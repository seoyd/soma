use std::path::PathBuf;

use soma_zero::{OfficialEvidenceExpansionConfig, ReasonCode};

fn example_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join(name)
}

#[test]
fn official_evidence_expansion_config_has_safe_defaults() {
    let config = OfficialEvidenceExpansionConfig::default();

    assert!(config.require_core_check);
    assert!(!config.run_external_eval);
    assert!(config.max_storage_bytes > 0);
}

#[test]
fn official_evidence_expansion_config_rejects_remote_paths() {
    let config = OfficialEvidenceExpansionConfig {
        output_root: "https://example.com/out".to_string(),
        ..OfficialEvidenceExpansionConfig::default()
    };

    assert!(
        config
            .validate_local_paths()
            .contains(&ReasonCode::RemotePathRejected)
    );
}

#[test]
fn official_evidence_expansion_config_surface_stays_research_only() {
    let toml = OfficialEvidenceExpansionConfig::default()
        .to_toml_string()
        .expect("serialize config");

    assert!(!toml.contains("broker"));
    assert!(!toml.contains("account"));
    assert!(!toml.contains("llm"));
}

#[test]
fn sprint25_official_evidence_expansion_examples_parse() {
    for path in [
        example_path("soma_official_evidence_expansion_crypto_only.toml"),
        example_path("soma_official_evidence_expansion_multi_venue.toml"),
    ] {
        let config = OfficialEvidenceExpansionConfig::from_toml_path(&path)
            .expect("parse expansion example");
        assert!(!config.expansion_id.is_empty());
        assert!(config.require_core_check);
    }
}

mod common;

use soma_zero::ReasonCode;

#[test]
fn valid_local_input_config_can_be_constructed() {
    let config = common::onboarding_config("local-config", "generic_ohlcv_valid_alt.csv");
    assert_eq!(
        config.validate_local_paths(),
        vec![ReasonCode::DeterministicPath]
    );
    assert!(config.build_provenance().user_supplied);
}

#[test]
fn remote_url_like_input_path_is_rejected() {
    let mut config = common::onboarding_config("remote-config", "generic_ohlcv_valid_alt.csv");
    config.input_path = "https://example.com/data.csv".to_string();
    assert!(
        config
            .validate_local_paths()
            .contains(&ReasonCode::LocalPathRejected)
    );
}

#[test]
fn source_label_generation_is_deterministic_and_has_no_live_fields() {
    let config = common::onboarding_config("deterministic-label", "generic_ohlcv_valid_alt.csv");
    let label_a = config.resolved_source_label();
    let label_b = config.resolved_source_label();
    let serialized = config.to_toml_string().expect("serialize config");
    assert_eq!(label_a, label_b);
    assert!(!serialized.to_ascii_lowercase().contains("broker"));
    assert!(!serialized.to_ascii_lowercase().contains("api"));
    assert!(!serialized.to_ascii_lowercase().contains("llm"));
}

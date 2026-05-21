mod common;

use soma_zero::{ConfigGenerationPolicy, generate_config_bundle};

#[test]
fn ready_preflight_generates_real_local_configs() {
    let config = common::onboarding_config("config-generation", "generic_ohlcv_valid_alt.csv");
    let report = common::run_preflight(&config);
    let bundle = generate_config_bundle(&config, &report, ConfigGenerationPolicy::ReadyOnly)
        .expect("bundle");
    assert!(
        bundle
            .dataset_entry_toml
            .contains("source_kind = \"RealLocal\"")
    );
    assert!(bundle.dataset_entry_toml.contains("user_supplied = true"));
    assert!(
        bundle
            .real_evidence_closure_toml
            .contains("generated_real_local_dataset_entry.toml")
    );
}

#[test]
fn generated_config_contains_no_live_api_broker_or_llm_fields() {
    let config = common::onboarding_config("config-safety", "generic_ohlcv_valid_alt.csv");
    let report = common::run_preflight(&config);
    let bundle = generate_config_bundle(&config, &report, ConfigGenerationPolicy::ReadyOnly)
        .expect("bundle");
    let joined = [
        bundle.dataset_entry_toml,
        bundle.real_evidence_closure_toml,
        bundle.batch_matrix_toml.unwrap_or_default(),
        bundle.ablation_study_toml.unwrap_or_default(),
    ]
    .join("\n")
    .to_ascii_lowercase();
    assert!(!joined.contains("broker"));
    assert!(!joined.contains("api"));
    assert!(!joined.contains("llm"));
}

#[test]
fn non_ready_preflight_only_generates_diagnostics_in_diagnostic_mode() {
    let mut config = common::onboarding_config("diagnostic-only", "generic_ohlcv_valid.csv");
    config.min_rows_for_preflight = 100;
    let report = common::run_preflight(&config);
    assert!(generate_config_bundle(&config, &report, ConfigGenerationPolicy::ReadyOnly).is_none());
    let bundle = generate_config_bundle(&config, &report, ConfigGenerationPolicy::DiagnosticOnly)
        .expect("diagnostic bundle");
    assert!(
        bundle
            .real_evidence_closure_toml
            .contains("diagnostic only")
    );
}

mod common;
#[path = "support/sprint64_support.rs"]
mod sprint64_support;

use std::fs;

use soma_zero::{
    ExternalArtifactRegistryAuditStatus, ExternalArtifactRegistryRunner, ExternalModelCardV2,
};

#[test]
fn safe_registry_passes_audit() {
    let config = sprint64_support::registry_config_from_example(
        "soma_external_registry_audit.toml",
        "audit-safe",
    );
    let report = ExternalArtifactRegistryRunner::default()
        .run_audit(&config)
        .expect("run audit");
    assert_eq!(
        report.audit_status,
        ExternalArtifactRegistryAuditStatus::Passed
    );
}

#[test]
fn unsafe_fields_and_secret_like_content_fail_audit() {
    let mut unsafe_config = sprint64_support::registry_config_from_example(
        "soma_external_registry_audit.toml",
        "audit-unsafe",
    );
    let card_path = sprint64_support::absolutize("examples/sprint64_data/model_card_a_v1.json");
    let mut unsafe_card: ExternalModelCardV2 =
        serde_json::from_str(&fs::read_to_string(card_path).expect("read model card"))
            .expect("parse model card");
    unsafe_card.intended_use = "Live broker execution".to_string();
    unsafe_card.live_use_forbidden = false;
    let unsafe_path =
        sprint64_support::write_support_json("audit-unsafe", "model_card_a_v1.json", &unsafe_card);
    unsafe_config.external_model_card_paths[0] = unsafe_path;
    let unsafe_report = ExternalArtifactRegistryRunner::default()
        .run_audit(&unsafe_config)
        .expect("run unsafe audit");
    assert_eq!(
        unsafe_report.audit_status,
        ExternalArtifactRegistryAuditStatus::FailedUnsafeFields
    );

    let mut secret_config = sprint64_support::registry_config_from_example(
        "soma_external_registry_audit.toml",
        "audit-secret",
    );
    let secret_text = fs::read_to_string(sprint64_support::absolutize(
        "examples/sprint64_data/model_card_a_v1.json",
    ))
    .expect("read source card")
    .replace("diagnostic-fixture", "api_key_fixture");
    let secret_path =
        sprint64_support::write_support_file("audit-secret", "model_card_a_v1.json", &secret_text);
    secret_config.external_model_card_paths[0] = secret_path;
    let secret_report = ExternalArtifactRegistryRunner::default()
        .run_audit(&secret_config)
        .expect("run secret audit");
    assert_eq!(
        secret_report.audit_status,
        ExternalArtifactRegistryAuditStatus::FailedSecretScan
    );
}

#[path = "support/sprint69_support.rs"]
mod support;

use std::fs;

use serde_json::Value;
use soma_zero::OfflineEvidenceAttachmentRegistryStatus;

#[test]
fn offline_evidence_attachment_bundle_matches_expected_summary_and_static_outputs() {
    let bundle = support::run_offline_attachment(
        "soma_offline_evidence_attach.toml",
        "offline-evidence-attachment-core",
    );

    let expected: Value = support::read_json(support::example_path(
        "sprint72_data/expected_attachment_summary.json",
    ));
    assert_eq!(
        expected["registry_status"],
        format!(
            "{:?}",
            bundle.offline_evidence_attachment_registry.registry_status
        )
    );
    assert_eq!(
        expected["valid_count"].as_u64().unwrap(),
        bundle.offline_evidence_attachment_registry.valid_count as u64
    );
    assert_eq!(
        expected["invalid_count"].as_u64().unwrap(),
        bundle.offline_evidence_attachment_registry.invalid_count as u64
    );
    assert_eq!(
        expected["research_only_count"].as_u64().unwrap(),
        bundle
            .offline_evidence_attachment_registry
            .research_only_count as u64
    );
    assert_eq!(
        expected["diagnostic_only_count"].as_u64().unwrap(),
        bundle
            .offline_evidence_attachment_registry
            .diagnostic_only_count as u64
    );
    assert_eq!(
        expected["official_count"].as_u64().unwrap(),
        bundle.offline_evidence_attachment_registry.official_count as u64
    );
    assert_eq!(
        expected["unknown_count"].as_u64().unwrap(),
        bundle.offline_evidence_attachment_registry.unknown_count as u64
    );
    assert_eq!(
        bundle.offline_evidence_attachment_registry.registry_status,
        OfflineEvidenceAttachmentRegistryStatus::AttachmentRegistryReady
    );

    let bundle_dir = support::attachment_output_path("offline-evidence-attachment-core")
        .join("sprint72-offline-evidence-attachment")
        .join("fragments");
    for name in [
        "evidence_attachment.html",
        "prediction_history_expansion.html",
        "retirement_evidence_pack.html",
        "owner_checklist_closure.html",
        "direct_watch_readiness.html",
    ] {
        let html = fs::read_to_string(bundle_dir.join(name)).expect("read sprint72 html fragment");
        assert!(html.contains("<section"));
        assert!(!html.contains("<form"));
        assert!(!html.contains("http://"));
        assert!(!html.contains("https://"));
    }
}

#[test]
fn offline_evidence_attachment_rejects_remote_paths_and_enforces_limits() {
    let mut remote = support::attachment_config_from_example(
        "soma_offline_evidence_attach.toml",
        "offline-evidence-attachment-remote-guard",
    );
    remote.kis_evidence_paths = vec!["https://example.com/evidence.json".to_string()];
    assert!(remote.validate().is_err());

    let mut bounded = support::attachment_config_from_example(
        "soma_offline_evidence_attach.toml",
        "offline-evidence-attachment-max-artifacts",
    );
    bounded.max_artifacts = 1;
    assert!(
        soma_zero::OfflineEvidenceAttachmentRunner::default()
            .run(&bounded)
            .is_err()
    );
}

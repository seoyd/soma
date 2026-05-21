#[path = "support/sprint69_support.rs"]
mod support;

use serde_json::Value;
use soma_zero::{
    ControlTowerRealEvidenceRefreshStatus, RealEvidenceAttachmentStatus, RealEvidenceFollowupConfig,
};

#[test]
fn config_defaults_are_conservative() {
    let config = RealEvidenceFollowupConfig::default();
    assert!(config.prefer_local_import);
    assert!(!config.allow_operator_market_data_collection);
    assert!(config.require_provenance);
    assert!(config.require_preflight);
    assert!(config.require_no_lookahead_safe);
}

#[test]
fn real_evidence_followup_bundle_matches_expected_example() {
    let bundle = support::run_sprint74_bundle(
        "soma_real_evidence_followup.toml",
        "real-evidence-followup-example",
    );
    let expected: Value = support::read_json(support::example_path(
        "sprint74_data/expected_real_evidence_followup.json",
    ));
    assert_eq!(
        format!(
            "{:?}",
            bundle.real_evidence_attachment_report.attachment_status
        ),
        expected["attachment_status"]
            .as_str()
            .expect("attachment status")
    );
    assert_eq!(
        format!(
            "{:?}",
            bundle.kis_real_evidence_validation_report.validation_status
        ),
        expected["validation_status"]
            .as_str()
            .expect("validation status")
    );
    assert_eq!(
        format!("{:?}", bundle.real_evidence_provenance_audit.audit_status),
        expected["provenance_status"]
            .as_str()
            .expect("provenance status")
    );
    assert_eq!(
        format!("{:?}", bundle.real_evidence_preflight_audit.audit_status),
        expected["preflight_status"]
            .as_str()
            .expect("preflight status")
    );
    assert_eq!(
        format!(
            "{:?}",
            bundle
                .real_evidence_outcome_readiness_report
                .readiness_status
        ),
        expected["outcome_status"].as_str().expect("outcome status")
    );
    assert_eq!(
        format!(
            "{:?}",
            bundle
                .real_evidence_sequence_readiness_report
                .readiness_status
        ),
        expected["sequence_status"]
            .as_str()
            .expect("sequence status")
    );
    assert_eq!(
        format!(
            "{:?}",
            bundle.real_evidence_model_ops_impact_report.impact_status
        ),
        expected["model_ops_status"]
            .as_str()
            .expect("model ops status")
    );
    assert_eq!(
        bundle.real_evidence_attachment_report.attachment_status,
        RealEvidenceAttachmentStatus::RealEvidenceAttached
    );
    assert_eq!(
        bundle.control_tower_real_evidence_refresh.refresh_status,
        ControlTowerRealEvidenceRefreshStatus::RealEvidenceRefreshReadyWithWarnings
    );
}

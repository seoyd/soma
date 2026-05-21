#[path = "support/sprint69_support.rs"]
mod support;

use serde_json::json;
use soma_zero::{RealEvidenceFollowupRunner, RealEvidenceProvenanceAuditStatus};

#[test]
fn provenance_present_passes() {
    let report = support::run_sprint74_bundle(
        "soma_real_provenance_audit.toml",
        "real-evidence-provenance-ok",
    )
    .real_evidence_provenance_audit;
    assert_eq!(
        report.audit_status,
        RealEvidenceProvenanceAuditStatus::ProvenanceReady
    );
    assert_eq!(report.remote_url_like_count, 0);
}

#[test]
fn remote_url_like_path_is_rejected() {
    let provenance_path = support::write_support_json(
        "real-evidence-provenance-remote",
        "provenance.json",
        &json!({"records":[{"local_path":"examples/sprint74_data/kis_real_canonical_sample.csv","provider_label":"KIS","source_class":"OfficialKIS","downloaded_by_soma": false,"remote_url":"https://example.com/file.csv"}]}),
    );
    let mut config = support::sprint74_config_from_example(
        "soma_real_provenance_audit.toml",
        "real-evidence-provenance-remote-config",
    );
    config.kis_provenance_paths = vec![provenance_path];
    let report = RealEvidenceFollowupRunner::default()
        .run_provenance_audit(&config)
        .expect("run provenance");
    assert_eq!(
        report.audit_status,
        RealEvidenceProvenanceAuditStatus::UnsafeRemoteSource
    );
}

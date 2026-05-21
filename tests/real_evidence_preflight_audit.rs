#[path = "support/sprint69_support.rs"]
mod support;

use serde_json::json;
use soma_zero::{RealEvidenceFollowupRunner, RealEvidencePreflightAuditStatus};

#[test]
fn preflight_present_passes() {
    let report = support::run_sprint74_bundle(
        "soma_real_preflight_audit.toml",
        "real-evidence-preflight-ok",
    )
    .real_evidence_preflight_audit;
    assert_eq!(
        report.audit_status,
        RealEvidencePreflightAuditStatus::PreflightReady
    );
    assert!(report.data_quality_avg >= 0.98);
}

#[test]
fn preflight_failure_is_detected() {
    let preflight_path = support::write_support_json(
        "real-evidence-preflight-fail",
        "preflight.json",
        &json!({"records":[{"local_path":"examples/sprint74_data/kis_real_canonical_sample.csv","passed":false,"data_quality_score":0.61}]}),
    );
    let mut config = support::sprint74_config_from_example(
        "soma_real_preflight_audit.toml",
        "real-evidence-preflight-fail-config",
    );
    config.kis_preflight_paths = vec![preflight_path];
    let report = RealEvidenceFollowupRunner::default()
        .run_preflight_audit(&config)
        .expect("run preflight");
    assert_eq!(
        report.audit_status,
        RealEvidencePreflightAuditStatus::PreflightFailed
    );
}

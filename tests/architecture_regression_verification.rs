mod support;

use soma_zero::{DualAgentWorkflowConfig, Sprint104DualAgentPaperLifecycleRunner};
use support::sprint104_support::write_manifest;

#[test]
fn architecture_regression_verification_preserves_member_owned_architecture() {
    let bundle = Sprint104DualAgentPaperLifecycleRunner::default()
        .run(&DualAgentWorkflowConfig::default())
        .expect("run");
    let report = bundle.architecture_regression_verification_report;
    assert_eq!(report.architecture_status, "ArchitectureVerified");
    assert!(report.committee_owned_core_preserved);
    assert!(report.member_owned_core_refs_preserved);
}

#[test]
fn architecture_regression_verification_detects_central_core_regression() {
    let manifest = write_manifest(
        "architecture-regression-verification",
        "changed_files.json",
        &["src/central_ai_core.rs"],
    );
    let mut config = DualAgentWorkflowConfig::default();
    config.changed_file_manifest_paths = Some(vec![manifest]);
    let bundle = Sprint104DualAgentPaperLifecycleRunner::default()
        .run(&config)
        .expect("run");
    assert_eq!(
        bundle
            .architecture_regression_verification_report
            .architecture_status,
        "ArchitectureRegressionDetected"
    );
}

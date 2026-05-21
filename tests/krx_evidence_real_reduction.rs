mod support;

use soma_zero::{
    CompileFamilyV2, KrxEvidenceRealReductionAction, KrxEvidenceRealReductionConfig,
    KrxEvidenceRealReductionStatus, RemainingBlockerQueueV7Status,
    Sprint91KrxEvidenceRecoveryRunner,
};
use support::sprint69_support as sprint;

#[test]
fn krx_evidence_config_defaults_stay_conservative() {
    let config = KrxEvidenceRealReductionConfig::default();
    assert_eq!(config.target_family, CompileFamilyV2::KrxEvidence);
    assert!(!config.run_real_no_run_after_reduction);
    assert!(!config.run_real_full_after_reduction);
    assert!(config.preserve_assertions);
    assert!(config.preserve_safety_guards);
    assert!(config.preserve_missing_auth_checks);
    assert!(config.preserve_endpoint_template_checks);
    assert!(config.preserve_market_data_only_checks);
    assert!(config.preserve_no_order_account_checks);
    assert!(config.preserve_source_boundary_checks);
    assert!(config.preserve_deterministic_status_checks);

    let value = serde_json::to_value(&config).expect("serialize");
    let object = value.as_object().expect("object");
    for forbidden in [
        "runtime",
        "training",
        "broker",
        "order",
        "account",
        "live_inference",
    ] {
        assert!(
            !object.contains_key(forbidden),
            "unexpected field {forbidden}"
        );
    }
}

#[test]
fn krx_evidence_config_rejects_remote_paths() {
    let config = KrxEvidenceRealReductionConfig {
        sprint90_bundle_paths: vec!["https://example.com/sprint90.json".to_string()],
        ..KrxEvidenceRealReductionConfig::default()
    };
    assert!(config.validate().is_err());
}

#[test]
fn krx_evidence_plan_keeps_suite_donors_and_actions() {
    let config = sprint::sprint91_config_from_example(
        "soma_krx_evidence_real_reduction_plan.toml",
        "krx-real-plan",
    );
    let plan = Sprint91KrxEvidenceRecoveryRunner::default()
        .run_krx_evidence_real_reduction_plan(&config)
        .expect("plan");
    assert_eq!(
        plan.target_files,
        vec!["tests/krx_evidence_suite.rs".to_string()]
    );
    assert!(
        plan.donor_files
            .contains(&"tests/krx_collection_dry_run.rs".to_string())
    );
    assert!(
        plan.donor_files
            .contains(&"tests/krx_evidence_job_plan.rs".to_string())
    );
    assert!(
        plan.actions
            .contains(&KrxEvidenceRealReductionAction::VerifyGroupedSuiteCoverage)
    );
    assert!(
        plan.actions
            .contains(&KrxEvidenceRealReductionAction::MoveRemainingAssertions)
    );
    assert!(
        plan.actions
            .contains(&KrxEvidenceRealReductionAction::ApplySharedFixtureHarness)
    );
    assert!(
        plan.actions
            .contains(&KrxEvidenceRealReductionAction::ReduceKrxAuthFixtureDuplication)
    );
    assert!(
        plan.actions
            .contains(&KrxEvidenceRealReductionAction::ReduceEndpointTemplateFixtureDuplication)
    );
    assert!(
        plan.actions
            .contains(&KrxEvidenceRealReductionAction::ReduceRawArchiveFixtureDuplication)
    );
}

#[test]
fn krx_evidence_bundle_stays_conservative_and_threads_statuses() {
    let bundle = sprint::run_sprint91_bundle(
        "soma_sprint91_krx_evidence_recover.toml",
        "krx-real-reduction-bundle",
    );
    assert_eq!(
        bundle.krx_evidence_real_reduction_report.reduction_status,
        KrxEvidenceRealReductionStatus::KrxEvidenceReducedWithWarnings
    );
    assert_eq!(
        bundle.remaining_blocker_queue_v7.queue_status,
        RemainingBlockerQueueV7Status::QueueReduced
    );
    assert_eq!(
        bundle
            .seven_blocker_queue_progress_report_v7
            .primary_next_family,
        "KrxEvidence"
    );
}

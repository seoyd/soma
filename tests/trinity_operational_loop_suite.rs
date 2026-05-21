#[path = "support/sprint59_support.rs"]
mod sprint59_support;

use std::path::Path;

use soma_zero::{
    SystemIntegrationReviewRunner, TrinityCommitteeOperationalLoopConfig,
    TrinityCommitteeReadinessStatus, TrinityOperationalLoopFinalStatus,
    TrinityOperationalLoopRunner,
};

#[test]
fn trinity_operational_loop_runs_and_counts_states() {
    let config = TrinityCommitteeOperationalLoopConfig::from_toml_path(Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/examples/soma_trinity_operational_loop_kis.toml"
    )))
    .expect("config");
    let bundle = TrinityOperationalLoopRunner::default()
        .run(&config)
        .expect("run loop");
    assert!(bundle.report.generated_candidate_count >= 4);
    assert!(bundle.report.cycle_count >= 1);
    assert!(matches!(
        bundle.report.final_status,
        TrinityOperationalLoopFinalStatus::OperationalLoopReady
            | TrinityOperationalLoopFinalStatus::PaperOnlyMonitoringReady
            | TrinityOperationalLoopFinalStatus::OwnerReviewPending
    ));
}

#[test]
fn trinity_operational_loop_preserves_exactly_three_active_personas() {
    let config = sprint59_support::review_config_from_example(
        "soma_system_review_full.toml",
        "trinity-operational-loop-suite",
    );
    let bundle = SystemIntegrationReviewRunner::default()
        .run(&config)
        .expect("run review");
    assert_eq!(
        bundle.trinity_readiness_report.readiness_status,
        TrinityCommitteeReadinessStatus::Ready
    );
    assert!(bundle.trinity_readiness_report.all_three_active);
    assert!(bundle.trinity_readiness_report.no_extra_active_personas);
}

#[test]
fn trinity_operational_loop_is_deterministic() {
    let config = TrinityCommitteeOperationalLoopConfig::from_toml_path(Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/examples/soma_trinity_operational_loop_kis.toml"
    )))
    .expect("config");
    let first = TrinityOperationalLoopRunner::default()
        .run(&config)
        .expect("first");
    let second = TrinityOperationalLoopRunner::default()
        .run(&config)
        .expect("second");
    assert_eq!(first.report.fingerprint, second.report.fingerprint);
    assert_eq!(
        first.operational_audit_timeline.fingerprint,
        second.operational_audit_timeline.fingerprint
    );
}

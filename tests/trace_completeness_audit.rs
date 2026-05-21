mod common;
#[path = "support/sprint69_support.rs"]
mod sprint69_support;

use soma_zero::{
    BaselineSnapshotCoverageRunner, TraceCompletenessAuditStatus, TraceCoverageMatrixStatus,
};

#[test]
fn trace_completeness_audit_and_scores_are_reported() {
    let bundle =
        sprint69_support::run_coverage("soma_trace_completeness_audit.toml", "trace-audit");
    assert_eq!(
        bundle.trace_completeness_audit.audit_status,
        TraceCompletenessAuditStatus::TraceCompleteWithWarnings
    );
    assert_eq!(
        bundle.trace_completeness_audit.trace_completeness_ratio,
        "0.9286"
    );
    assert_eq!(bundle.trace_completeness_audit.complete_model_versions, 1);
    assert_eq!(
        bundle.trace_coverage_matrix.matrix_status,
        TraceCoverageMatrixStatus::PartialCoverage
    );
    assert_eq!(
        bundle.per_model_trace_completeness_report.average_score,
        "0.9286"
    );
    assert_eq!(bundle.per_model_trace_completeness_report.complete_count, 1);
    assert_eq!(
        bundle
            .per_model_trace_completeness_report
            .critical_missing_count,
        0
    );
}

#[test]
fn removing_baselines_marks_missing_baseline_trace() {
    let mut config = sprint69_support::coverage_config_from_example(
        "soma_trace_completeness_audit.toml",
        "trace-audit-missing-baseline",
    );
    config.baseline_snapshot_paths.clear();
    let audit = BaselineSnapshotCoverageRunner::default()
        .run_trace_completeness_audit(&config)
        .expect("run audit");
    assert_eq!(
        audit.audit_status,
        TraceCompletenessAuditStatus::MissingBaselineTrace
    );
}

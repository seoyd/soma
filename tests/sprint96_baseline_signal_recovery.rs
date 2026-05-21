mod support;

use soma_zero::{
    BaselineSignalCompileImpactStatus, BaselineSignalRealReductionStatus,
    BaselineSignalResearchOnlyStatus, CounterfactualBackfillEntryGateStatus,
    CounterfactualBackfillReadinessPrecheckStatus, RemainingBlockerQueueV12Status,
    SafetyCoveragePreservationReportV12Status, SevenBlockerQueueProgressStatusV12,
};
use support::sprint69_support as sprint;

fn bundle() -> soma_zero::Sprint96BaselineSignalRecoveryBundle {
    sprint::run_sprint96_bundle(
        "soma_sprint96_baseline_signal_recover.toml",
        "sprint96-baseline-signal-recovery",
    )
}

#[test]
fn baseline_signal_real_reduction_preserves_conservative_semantics() {
    let bundle = bundle();
    assert_eq!(
        bundle
            .baseline_signal_real_reduction_report
            .reduction_status,
        BaselineSignalRealReductionStatus::BaselineSignalReducedWithWarnings
    );
    assert_eq!(
        bundle
            .baseline_signal_research_only_preservation_report
            .research_status,
        BaselineSignalResearchOnlyStatus::ResearchOnlyPreserved
    );
    assert_eq!(
        bundle.baseline_signal_compile_impact_report.impact_status,
        BaselineSignalCompileImpactStatus::CompileImpactSampleBacked
    );
}

#[test]
fn counterfactual_backfill_entry_gate_and_precheck_are_ready() {
    let bundle = bundle();
    assert_eq!(
        bundle.counterfactual_backfill_entry_gate.gate_status,
        CounterfactualBackfillEntryGateStatus::CounterfactualBackfillEntryReady
    );
    assert_eq!(
        bundle
            .counterfactual_backfill_readiness_precheck_report
            .precheck_status,
        CounterfactualBackfillReadinessPrecheckStatus::CounterfactualBackfillPrecheckReady
    );
}

#[test]
fn sprint96_queue_workspace_and_safety_reports_stay_consistent() {
    let bundle = bundle();
    assert_eq!(
        bundle.seven_blocker_queue_progress_report_v12.queue_status,
        SevenBlockerQueueProgressStatusV12::QueueAdvanced
    );
    assert_eq!(
        bundle.remaining_blocker_queue_v12.queue_status,
        RemainingBlockerQueueV12Status::QueueAdvanced
    );
    assert_eq!(
        bundle.safety_coverage_preservation_report_v12.safety_status,
        SafetyCoveragePreservationReportV12Status::SafetyCoveragePreserved
    );
    assert_eq!(
        bundle.remaining_blocker_queue_v12.primary_next_family,
        "CounterfactualBackfill"
    );
}

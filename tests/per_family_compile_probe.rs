mod support;

use soma_zero::{
    PerFamilyCompileProbeStatus, PerFamilyExecutionProbeStatus, PerFamilyNoRunProbeStatus,
    Sprint88SevenBlockerRecoveryRunner,
};
use support::sprint69_support as sprint;

#[test]
fn per_family_compile_probe_covers_all_report_status_shapes() {
    let config =
        sprint::sprint88_config_from_example("soma_per_family_compile_probe.toml", "compile-probe");
    let reports = Sprint88SevenBlockerRecoveryRunner::default()
        .run_per_family_compile_probe(&config)
        .expect("compile probes");
    let statuses = reports
        .iter()
        .map(|report| report.probe_status)
        .collect::<Vec<_>>();
    assert!(statuses.contains(&PerFamilyCompileProbeStatus::SampleBacked));
    assert!(statuses.contains(&PerFamilyCompileProbeStatus::ProbePassed));
    assert!(statuses.contains(&PerFamilyCompileProbeStatus::ProbePassedWithWarnings));
    assert!(statuses.contains(&PerFamilyCompileProbeStatus::ProbeStillBlocked));
    assert!(statuses.contains(&PerFamilyCompileProbeStatus::ProbeFailed));
    assert!(statuses.contains(&PerFamilyCompileProbeStatus::NotRun));
}

#[test]
fn per_family_no_run_probe_stays_distinct_from_full_workspace_acceptance() {
    let config =
        sprint::sprint88_config_from_example("soma_per_family_no_run_probe.toml", "no-run-probe");
    let reports = Sprint88SevenBlockerRecoveryRunner::default()
        .run_per_family_no_run_probe(&config)
        .expect("no run probes");
    let statuses = reports
        .iter()
        .map(|report| report.no_run_status)
        .collect::<Vec<_>>();
    assert!(statuses.contains(&PerFamilyNoRunProbeStatus::NoRunPassed));
    assert!(statuses.contains(&PerFamilyNoRunProbeStatus::NoRunStillBlocked));
    assert!(statuses.contains(&PerFamilyNoRunProbeStatus::NoRunFailed));
    assert!(statuses.contains(&PerFamilyNoRunProbeStatus::NotRun));
    assert!(statuses.contains(&PerFamilyNoRunProbeStatus::SampleBacked));
}

#[test]
fn per_family_execution_probe_is_deterministic() {
    let config = sprint::sprint88_config_from_example(
        "soma_per_family_execution_probe.toml",
        "execution-probe",
    );
    let first = Sprint88SevenBlockerRecoveryRunner::default()
        .run_per_family_execution_probe(&config)
        .expect("first");
    let second = Sprint88SevenBlockerRecoveryRunner::default()
        .run_per_family_execution_probe(&config)
        .expect("second");
    let statuses = first
        .iter()
        .map(|report| report.execution_status)
        .collect::<Vec<_>>();
    assert!(statuses.contains(&PerFamilyExecutionProbeStatus::ExecutionPassed));
    assert!(statuses.contains(&PerFamilyExecutionProbeStatus::ExecutionStillBlocked));
    assert!(statuses.contains(&PerFamilyExecutionProbeStatus::ExecutionFailed));
    assert!(statuses.contains(&PerFamilyExecutionProbeStatus::NotRun));
    assert!(statuses.contains(&PerFamilyExecutionProbeStatus::SampleBacked));
    assert_eq!(first, second);
}

mod support;

use soma_zero::{
    KrxEvidenceFullGateRerunStatus, KrxEvidenceNoRunGateRerunStatus,
    Sprint91KrxEvidenceRecoveryRunner,
};
use std::fs;
use support::{shared_fixture_harness as harness, sprint69_support as sprint};

#[test]
fn krx_no_run_and_full_reruns_are_not_run_by_default() {
    let config = sprint::sprint91_config_from_example(
        "soma_krx_evidence_no_run_rerun.toml",
        "krx-gate-rerun-default",
    );
    let no_run = Sprint91KrxEvidenceRecoveryRunner::default()
        .run_krx_evidence_no_run_rerun(&config)
        .expect("no-run");
    let full = Sprint91KrxEvidenceRecoveryRunner::default()
        .run_krx_evidence_full_gate_rerun(&config)
        .expect("full");
    assert_eq!(no_run.status, KrxEvidenceNoRunGateRerunStatus::NotRun);
    assert_eq!(full.status, KrxEvidenceFullGateRerunStatus::NotRun);
}

#[test]
fn krx_no_run_and_full_reruns_support_recovered_still_blocked_and_failed() {
    let mut config = sprint::sprint91_config_from_example(
        "soma_krx_evidence_no_run_rerun.toml",
        "krx-gate-rerun-custom",
    );
    config.run_real_no_run_after_reduction = true;
    config.run_real_full_after_reduction = true;
    let dir = harness::temp_output_dir_for_test("krx-gate-rerun-custom");

    let no_run_path = dir.join("krx_no_run_rerun_sample.json");
    fs::write(
        &no_run_path,
        serde_json::to_string_pretty(&serde_json::json!({
            "started": true,
            "finished": true,
            "passed": true,
            "duration_ms": 12,
            "remaining_blocker_families": []
        }))
        .expect("json"),
    )
    .expect("write no-run sample");
    let full_path = dir.join("krx_full_gate_rerun_sample.json");
    fs::write(
        &full_path,
        serde_json::to_string_pretty(&serde_json::json!({
            "started": true,
            "finished": true,
            "passed": false,
            "duration_ms": 30,
            "remaining_blocker_families": []
        }))
        .expect("json"),
    )
    .expect("write full sample");
    config.cargo_metadata_paths.retain(|value| {
        !value.ends_with("krx_no_run_rerun_sample.json")
            && !value.ends_with("krx_full_gate_rerun_sample.json")
    });
    config.cargo_metadata_paths.extend([
        no_run_path.display().to_string(),
        full_path.display().to_string(),
    ]);

    let no_run = Sprint91KrxEvidenceRecoveryRunner::default()
        .run_krx_evidence_no_run_rerun(&config)
        .expect("no-run");
    let full = Sprint91KrxEvidenceRecoveryRunner::default()
        .run_krx_evidence_full_gate_rerun(&config)
        .expect("full");
    assert_eq!(
        no_run.status,
        KrxEvidenceNoRunGateRerunStatus::NoRunRecoveredAfterKrxEvidence
    );
    assert_eq!(
        full.status,
        KrxEvidenceFullGateRerunStatus::FullWorkspaceFailedAfterKrxEvidence
    );
}

#[test]
fn krx_no_run_does_not_imply_full_acceptance() {
    let mut config = sprint::sprint91_config_from_example(
        "soma_krx_evidence_no_run_rerun.toml",
        "krx-gate-rerun-no-run-only",
    );
    config.run_real_no_run_after_reduction = true;
    let no_run_path = sprint::write_support_json(
        "krx-gate-rerun-no-run-only",
        "krx_no_run_rerun_sample.json",
        &serde_json::json!({
            "started": true,
            "finished": true,
            "passed": true,
            "duration_ms": 12,
            "remaining_blocker_families": []
        }),
    );
    config
        .cargo_metadata_paths
        .retain(|value| !value.ends_with("krx_no_run_rerun_sample.json"));
    config.cargo_metadata_paths.push(no_run_path);
    let bundle = Sprint91KrxEvidenceRecoveryRunner::default()
        .run(&config)
        .expect("bundle");
    assert_eq!(
        bundle.krx_evidence_no_run_gate_rerun_report.status,
        KrxEvidenceNoRunGateRerunStatus::NoRunRecoveredAfterKrxEvidence
    );
    assert_eq!(
        bundle.krx_evidence_full_gate_rerun_report.status,
        KrxEvidenceFullGateRerunStatus::NotRun
    );
}

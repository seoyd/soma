mod common;
#[path = "support/sprint62_support.rs"]
mod sprint62_support;

use serde_json::json;
use soma_zero::{Mamba3FinExternalPrototypeGateStatus, SequenceDatasetExportRunner};

#[test]
fn mamba_external_gate_is_planning_ready() {
    let config = sprint62_support::export_config_from_example(
        "soma_mamba3fin_prototype_gate.toml",
        "mamba-gate-ready",
    );
    let report = SequenceDatasetExportRunner::default()
        .run_mamba3fin_prototype_gate(&config)
        .expect("run mamba gate");
    assert_eq!(
        report.gate_status,
        Mamba3FinExternalPrototypeGateStatus::PlanningReady
    );
    assert!(!report.rust_runtime_allowed);
    assert!(!report.training_allowed);
    assert!(!report.live_inference_allowed);
}

#[test]
fn mamba_external_gate_blocks_on_no_lookahead() {
    let mut config = sprint62_support::export_config_from_example(
        "soma_mamba3fin_prototype_gate.toml",
        "mamba-gate-blocked",
    );
    let path = sprint62_support::write_support_json(
        "mamba-gate-blocked",
        "no_lookahead_bad.json",
        &json!({
            "checked_windows": 6,
            "failed_windows": 2,
            "violation_examples": ["future leakage"]
        }),
    );
    config.no_lookahead_proof_paths = vec![path];
    let report = SequenceDatasetExportRunner::default()
        .run_mamba3fin_prototype_gate(&config)
        .expect("run blocked gate");
    assert_eq!(
        report.gate_status,
        Mamba3FinExternalPrototypeGateStatus::BlockedByNoLookahead
    );
}

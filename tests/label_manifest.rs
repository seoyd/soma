mod common;
#[path = "support/sprint62_support.rs"]
mod sprint62_support;

use serde_json::json;
use soma_zero::SequenceDatasetExportRunner;

#[test]
fn frozen_label_manifest_is_accepted() {
    let config = sprint62_support::export_config_from_example(
        "soma_sequence_dataset_export_small.toml",
        "label-frozen",
    );
    let bundle = SequenceDatasetExportRunner::default()
        .run(&config)
        .expect("run export");
    assert!(bundle.label_manifest.frozen);
}

#[test]
fn horizon_mismatch_blocks_rows() {
    let mut config = sprint62_support::export_config_from_example(
        "soma_sequence_dataset_export_small.toml",
        "label-mismatch",
    );
    let path = sprint62_support::write_support_json(
        "label-mismatch",
        "label_manifest_bad.json",
        &json!({
            "label_kinds": ["TakeProfit", "StopLoss"],
            "horizon_bars": [16],
            "barrier_profile_id": "bad",
            "cost_bps": 3.0,
            "slippage_bps": 2.0,
            "tie_break_policy": "deterministic-priority",
            "label_timestamp_policy": "strictly-after-window-end",
            "no_trade_counterfactual_policy": "bounded",
            "risk_denied_counterfactual_policy": "final",
            "version": "v1",
            "frozen": true
        }),
    );
    config.label_alignment_audit_paths = vec![path];
    let bundle = SequenceDatasetExportRunner::default()
        .run(&config)
        .expect("run mismatched export");
    assert_eq!(bundle.sequence_dataset_export_artifact.row_count, 0);
}

mod common;
#[path = "support/sprint61_support.rs"]
mod sprint61_support;

use serde_json::json;
use soma_zero::{
    BoundedKISOfficialEvidenceClosureRunner, NoLookaheadProofStatus,
    SequenceReadinessHardeningRecommendation, SequenceReadinessHardeningStatus,
};

#[test]
fn sequence_readiness_example_is_ready_for_export() {
    let config = sprint61_support::sequence_config_from_example(
        "soma_sequence_readiness_hardening.toml",
        "sequence-ready",
    );
    let report = BoundedKISOfficialEvidenceClosureRunner::default()
        .run_sequence_readiness_hardening(&config)
        .expect("run sequence hardening");
    assert_eq!(
        report.readiness_status,
        SequenceReadinessHardeningStatus::ReadyForSequenceDatasetExport
    );
    assert_eq!(
        report.final_recommendation,
        SequenceReadinessHardeningRecommendation::ExportSmallSequenceDataset
    );
}

#[test]
fn no_lookahead_proof_detects_violation() {
    let mut config = sprint61_support::sequence_config_from_example(
        "soma_no_lookahead_sequence_proof.toml",
        "sequence-violation",
    );
    let path = sprint61_support::write_support_json(
        "sequence-violation",
        "no_lookahead_sequence_violation.json",
        &json!({
            "checked_windows": 10,
            "passed_windows": 7,
            "failed_windows": 3,
            "violation_examples": ["future close leaked into feature vector"],
            "no_trade_depth": 3,
            "risk_denied_depth": 1,
            "no_lookahead_safe": false
        }),
    );
    config.counterfactual_paths = vec![path];
    let report = BoundedKISOfficialEvidenceClosureRunner::default()
        .build_no_lookahead_sequence_proof(&config)
        .expect("build proof");
    assert_eq!(
        report.proof_status,
        NoLookaheadProofStatus::NoLookaheadViolation
    );
}

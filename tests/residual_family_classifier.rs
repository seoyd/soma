mod support;

use soma_zero::{ResidualIntegrationFamily, Sprint86ResidualGateRecoveryRunner};
use support::sprint69_support as sprint;

#[test]
fn residual_family_classifier_maps_named_binaries_deterministically() {
    let config = sprint::sprint86_config_from_example(
        "soma_residual_family_classifier.toml",
        "residual-family-classifier-test",
    );
    let report = Sprint86ResidualGateRecoveryRunner::default()
        .run_residual_family_classifier(&config)
        .expect("classifier");
    assert!(report.classified_records.iter().any(|record| {
        record.binary_name.ends_with("committee_replay.rs")
            && record.family == ResidualIntegrationFamily::CommitteeScenarioReplay
    }));
    assert!(report.classified_records.iter().any(|record| {
        record.binary_name.ends_with("model_ops_operator_qa.rs")
            && record.family == ResidualIntegrationFamily::ModelOpsQA
    }));
    assert!(report.unknown_records.is_empty());
}

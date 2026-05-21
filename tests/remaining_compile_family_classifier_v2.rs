mod support;

use soma_zero::{CompileFamilyV2, Sprint87CompileGateRecoveryRunner};
use support::sprint69_support as sprint;

#[test]
fn remaining_compile_family_classifier_v2_maps_all_sample_families() {
    let config = sprint::sprint87_config_from_example(
        "soma_compile_family_classifier_v2.toml",
        "compile-family-classifier-v2",
    );
    let report = Sprint87CompileGateRecoveryRunner::default()
        .run_compile_family_classifier_v2(&config)
        .expect("classifier");
    assert!(report.classified_records.iter().any(|record| {
        record
            .binary_name
            .ends_with("future_window_requirements.rs")
            && record.family == CompileFamilyV2::FutureWindow
    }));
    assert!(report.classified_records.iter().any(|record| {
        record
            .binary_name
            .ends_with("official_evidence_diversity_gap.rs")
            && record.family == CompileFamilyV2::OfficialDiversity
    }));
    assert!(report.classified_records.iter().any(|record| {
        record.binary_name == "committee_cli_safety"
            && record.family == CompileFamilyV2::CommitteeCliSafety
    }));
    assert!(report.unknown_records.is_empty());
}

#[path = "support/sprint48_support.rs"]
mod support;

use soma_zero::{
    OfficialEvidenceDiversityGapKind, OfficialEvidenceDiversityGapRunner,
    OfficialEvidenceDiversityGapStatus,
};

#[test]
fn official_diversity_detects_outcome_gaps() {
    let config = support::diversity_gap_config("official-diversity-gap-suite");
    let report = OfficialEvidenceDiversityGapRunner::default()
        .run(&config)
        .expect("gap map");
    assert_eq!(
        report.gap_status,
        OfficialEvidenceDiversityGapStatus::SingleOutcomeDominated
    );

    let config = support::diversity_gap_config("official-diversity-missing-time-expired-suite");
    let report = OfficialEvidenceDiversityGapRunner::default()
        .run(&config)
        .expect("gap map");
    assert!(report.cells.iter().any(|cell| {
        cell.gap_kind == OfficialEvidenceDiversityGapKind::MissingTimeExpiredOutcomes
    }));
}

#[test]
fn official_diversity_preserves_source_boundary_and_diagnostic_only_rejection() {
    let mut config = support::diversity_gap_config("official-diversity-crypto-only-suite");
    config.multi_row_official_set_paths = vec![
        support::repo_path("examples/sprint48_data/diversity_multi_row_set_crypto.json")
            .display()
            .to_string(),
    ];
    config.batch_outcome_linkage_paths = vec![
        support::repo_path("examples/sprint48_data/diversity_crypto_outcomes.json")
            .display()
            .to_string(),
    ];
    config.batch_counterfactual_completion_paths = vec![
        support::repo_path("examples/sprint48_data/diversity_crypto_counterfactuals.json")
            .display()
            .to_string(),
    ];
    let report = OfficialEvidenceDiversityGapRunner::default()
        .run(&config)
        .expect("crypto gap map");
    assert_eq!(
        report.gap_status,
        OfficialEvidenceDiversityGapStatus::DiagnosticOnly
    );
}

#[test]
fn official_diversity_report_is_deterministic() {
    let config = support::diversity_gap_config("official-diversity-deterministic-suite");
    let first = OfficialEvidenceDiversityGapRunner::default()
        .run(&config)
        .expect("first");
    let second = OfficialEvidenceDiversityGapRunner::default()
        .run(&config)
        .expect("second");
    assert_eq!(first.to_text(), second.to_text());
}

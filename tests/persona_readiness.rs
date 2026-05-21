mod common;

use soma_zero::BatchExperimentRunner;

#[test]
fn persona_readiness_tracks_current_personas_and_stays_conservative() {
    let matrix = common::batch_matrix(
        "persona-readiness",
        vec![common::dataset_entry(
            "valid",
            "generic_ohlcv_valid.csv",
            true,
        )],
        vec![common::baseline_variant("baseline_5m", true)],
    );
    let report = BatchExperimentRunner::default().run_matrix(&matrix);

    let persona = &report.persona_readiness_summary;
    let union_count = persona
        .selected_vote_counts
        .keys()
        .chain(persona.forced_contrarian_counts.keys())
        .collect::<std::collections::BTreeSet<_>>()
        .len();
    assert_eq!(persona.current_persona_count, union_count);
    assert!(!persona.expansion_recommended);
}

#[test]
fn poor_data_keeps_persona_expansion_recommendation_disabled() {
    let matrix = common::batch_matrix(
        "persona-poor-data",
        vec![common::dataset_entry(
            "bad",
            "generic_ohlcv_bad_ohlc.csv",
            true,
        )],
        vec![common::baseline_variant("baseline_5m", true)],
    );

    let report = BatchExperimentRunner::default().run_matrix(&matrix);
    assert!(!report.persona_readiness_summary.expansion_recommended);
}

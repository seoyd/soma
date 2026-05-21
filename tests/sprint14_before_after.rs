use soma_zero::{
    ReasonCode, Sprint14ComparableSummary, Sprint14Track, build_before_after_report,
    sprint14_before_after_to_text,
};

fn summary() -> Sprint14ComparableSummary {
    Sprint14ComparableSummary {
        study_id: "study".to_string(),
        selected_track: Some(Sprint14Track::NeedMoreExperiments),
        dataset_count: 2,
        usable_dataset_count: 2,
        total_outcome_records: 0,
        comparable_variant_count: 0,
        average_data_quality_score: 0.87,
        no_runtime_llm: true,
        no_live_api: true,
        no_real_broker: true,
        no_real_order_execution: true,
        no_new_personas: true,
        reason_codes: vec![ReasonCode::DeterministicPath],
    }
}

#[test]
fn before_after_report_is_deterministic() {
    let report_a = build_before_after_report(summary(), summary());
    let report_b = build_before_after_report(summary(), summary());
    assert_eq!(report_a, report_b);
    assert_eq!(
        sprint14_before_after_to_text(&report_a),
        sprint14_before_after_to_text(&report_b)
    );
}

#[test]
fn before_after_report_flags_safety_regression() {
    let mut after = summary();
    after.no_live_api = false;
    let report = build_before_after_report(summary(), after);
    assert!(
        report
            .safety_regressions
            .contains(&"no_live_api_regressed".to_string())
    );
}

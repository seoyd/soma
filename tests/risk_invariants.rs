use soma_zero::build_risk_invariant_report;

#[test]
fn risk_invariant_report_covers_required_denials_and_is_deterministic() {
    let left = build_risk_invariant_report();
    let right = build_risk_invariant_report();

    assert!(left.default_deny_passed);
    assert!(left.missing_stop_denied);
    assert!(left.negative_edge_denied);
    assert!(left.low_data_quality_denied);
    assert!(left.invalid_prediction_denied);
    assert!(left.schema_mismatch_denied);
    assert!(left.emergency_stop_blocks_all);
    assert!(left.cooldown_blocks_new_entries);
    assert!(left.external_model_cannot_bypass);
    assert_eq!(left.to_text(), right.to_text());
}

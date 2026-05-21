use soma_zero::{
    TimeframeAlignmentInput, TimeframeAlignmentOverallStatus, TimeframeAlignmentStatus,
    build_timeframe_alignment_report,
};

#[test]
fn timeframe_alignment_reports_exact_aggregation_and_missing_metadata() {
    let exact = build_timeframe_alignment_report(
        &[TimeframeAlignmentInput {
            scenario_row_id: "row-1".to_string(),
            scenario_timeframe: "1d".to_string(),
            candle_series_id: "series-1".to_string(),
            candle_timeframe: "1d".to_string(),
        }],
        false,
        true,
    );
    assert_eq!(
        exact.records[0].status,
        TimeframeAlignmentStatus::ExactMatch
    );
    assert_eq!(
        exact.alignment_status,
        TimeframeAlignmentOverallStatus::HealthyTimeframeAlignment
    );

    let needs_permission = build_timeframe_alignment_report(
        &[TimeframeAlignmentInput {
            scenario_row_id: "row-2".to_string(),
            scenario_timeframe: "1d".to_string(),
            candle_series_id: "series-2".to_string(),
            candle_timeframe: "1h".to_string(),
        }],
        false,
        true,
    );
    assert_eq!(
        needs_permission.records[0].status,
        TimeframeAlignmentStatus::CompatibleAggregation
    );
    assert_eq!(
        needs_permission.alignment_status,
        TimeframeAlignmentOverallStatus::NeedsAggregationPermission
    );

    let allowed = build_timeframe_alignment_report(
        &[TimeframeAlignmentInput {
            scenario_row_id: "row-3".to_string(),
            scenario_timeframe: "1d".to_string(),
            candle_series_id: "series-3".to_string(),
            candle_timeframe: "1h".to_string(),
        }],
        true,
        true,
    );
    assert_eq!(
        allowed.records[0].status,
        TimeframeAlignmentStatus::CompatibleAggregation
    );
    assert_eq!(
        allowed.alignment_status,
        TimeframeAlignmentOverallStatus::HealthyTimeframeAlignment
    );

    let missing = build_timeframe_alignment_report(
        &[TimeframeAlignmentInput {
            scenario_row_id: "row-4".to_string(),
            scenario_timeframe: "unknown".to_string(),
            candle_series_id: "series-4".to_string(),
            candle_timeframe: "1d".to_string(),
        }],
        false,
        true,
    );
    assert_eq!(
        missing.records[0].status,
        TimeframeAlignmentStatus::MissingScenarioTimeframe
    );
    assert_eq!(
        missing.alignment_status,
        TimeframeAlignmentOverallStatus::InsufficientTimeframeMetadata
    );
}

#[test]
fn timeframe_alignment_rejects_upsample_and_is_deterministic() {
    let first = build_timeframe_alignment_report(
        &[TimeframeAlignmentInput {
            scenario_row_id: "row-1".to_string(),
            scenario_timeframe: "1h".to_string(),
            candle_series_id: "series-1".to_string(),
            candle_timeframe: "1d".to_string(),
        }],
        false,
        false,
    );
    let second = build_timeframe_alignment_report(
        &[TimeframeAlignmentInput {
            scenario_row_id: "row-1".to_string(),
            scenario_timeframe: "1h".to_string(),
            candle_series_id: "series-1".to_string(),
            candle_timeframe: "1d".to_string(),
        }],
        false,
        false,
    );
    assert_eq!(
        first.records[0].status,
        TimeframeAlignmentStatus::IncompatibleUpsample
    );
    assert_eq!(first.to_text(), second.to_text());
}

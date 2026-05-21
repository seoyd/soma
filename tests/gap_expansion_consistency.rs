use soma_zero::{
    CandleAcquisitionJob, CandleAcquisitionJobKind, CandleAcquisitionJobStatus,
    CandleAcquisitionPlan, CandleExpansionCounts, GapExpansionConsistencyStatus,
    OfficialCandleExpansionFinalStatus, OfficialCandleExpansionRecommendation,
    OfficialCandleExpansionReport, OfficialCandleGapCell, OfficialCandleGapKind,
    OfficialCandleGapStatus, ProviderMarket, RowCandleCandidateReport,
    RowCandleCandidateReportStatus, build_gap_expansion_consistency_report,
};

fn sample_gap_map(status: OfficialCandleGapStatus) -> soma_zero::OfficialCandleCoverageGapMap {
    soma_zero::OfficialCandleCoverageGapMap {
        gap_id: "gap".to_string(),
        cells: vec![OfficialCandleGapCell {
            market: ProviderMarket::USEquity,
            symbol: "AAPL".to_string(),
            normalized_symbol: "AAPL".to_string(),
            venue: None,
            timeframe: "1d".to_string(),
            horizon_bars: 3,
            source_kind: Some("OfficialNonCrypto".to_string()),
            source_class: soma_zero::ComparableEvidenceSourceClass::OfficialNonCrypto,
            row_count_impacted: 1,
            comparable_rows_impacted: 1,
            missing_future_bars: 3,
            required_start_timestamp_ms: Some(1),
            required_end_timestamp_ms: Some(2),
            required_min_rows: 4,
            gap_kinds: vec![OfficialCandleGapKind::MissingOfficialCandleSeries],
            buildable_from_existing_local_csv: false,
            buildable_from_provider_collection: true,
            requires_operator_action: true,
            related_artifact_paths: Vec::new(),
            reason_codes: Vec::new(),
        }],
        total_gaps: 1,
        official_gap_count: 1,
        non_crypto_official_gap_count: 1,
        crypto_gap_count: 0,
        diagnostic_gap_count: 0,
        research_only_gap_count: 0,
        fixture_gap_count: 0,
        buildable_gap_count: 0,
        operator_action_gap_count: 1,
        gap_status: status,
        warnings: Vec::new(),
        reason_codes: Vec::new(),
    }
}

fn sample_expansion(
    symbol: &str,
    final_status: OfficialCandleExpansionFinalStatus,
    added_series: usize,
) -> OfficialCandleExpansionReport {
    OfficialCandleExpansionReport {
        expansion_id: "exp".to_string(),
        gap_map: sample_gap_map(OfficialCandleGapStatus::MissingOfficialCandles),
        acquisition_plan: CandleAcquisitionPlan::from_jobs(
            "plan".to_string(),
            vec![CandleAcquisitionJob {
                job_id: "job".to_string(),
                job_kind: CandleAcquisitionJobKind::LocalOfficialCsvImport,
                provider_kind: None,
                market: "USEquity".to_string(),
                symbol: symbol.to_string(),
                venue: None,
                timeframe: "1d".to_string(),
                horizon_bars: 3,
                start_timestamp_ms: None,
                end_timestamp_ms: None,
                max_rows: 4,
                max_requests: 1,
                output_root: "target".to_string(),
                local_input_csv_path: None,
                local_input_provenance_path: None,
                local_input_preflight_path: None,
                local_input_manifest_path: None,
                expected_canonical_csv_path: None,
                expected_provenance_path: None,
                expected_preflight_path: None,
                status: CandleAcquisitionJobStatus::Planned,
                reason_codes: Vec::new(),
            }],
            Vec::new(),
            soma_zero::StorageBudgetReport::default(),
            Vec::new(),
            Vec::new(),
        ),
        executed_jobs: vec![CandleAcquisitionJob {
            job_id: "job".to_string(),
            job_kind: CandleAcquisitionJobKind::LocalOfficialCsvImport,
            provider_kind: None,
            market: "USEquity".to_string(),
            symbol: symbol.to_string(),
            venue: None,
            timeframe: "1d".to_string(),
            horizon_bars: 3,
            start_timestamp_ms: None,
            end_timestamp_ms: None,
            max_rows: 4,
            max_requests: 1,
            output_root: "target".to_string(),
            local_input_csv_path: None,
            local_input_provenance_path: None,
            local_input_preflight_path: None,
            local_input_manifest_path: None,
            expected_canonical_csv_path: None,
            expected_provenance_path: None,
            expected_preflight_path: None,
            status: CandleAcquisitionJobStatus::Planned,
            reason_codes: Vec::new(),
        }],
        new_candle_pack: None,
        backfill_report: None,
        reference_generation_summary: None,
        counterfactual_depth_summary: None,
        core_scorecard_rerun_summary: None,
        before_counts: Some(CandleExpansionCounts::default()),
        after_counts: CandleExpansionCounts {
            gap_count: 1,
            ..CandleExpansionCounts::default()
        },
        added_official_candle_series: added_series,
        added_non_crypto_official_candle_series: added_series,
        added_official_ready_matches: 0,
        added_backfilled_rows: 0,
        added_complete_comparable_rows: 0,
        added_outcome_references: 0,
        added_no_trade_counterfactuals: 0,
        added_risk_denied_counterfactuals: 0,
        previous_primary_bottleneck: None,
        current_primary_bottleneck: None,
        bottleneck_changed: false,
        final_status,
        final_recommendation: OfficialCandleExpansionRecommendation::NeedMoreEvidence,
        blockers: Vec::new(),
        warnings: Vec::new(),
        reason_codes: Vec::new(),
    }
}

#[test]
fn gap_expansion_consistency_detects_no_gap_mismatch_added_series_without_matches_and_job_target_mismatch()
 {
    let candidate_report = RowCandleCandidateReport {
        candidates_by_row: Vec::new(),
        rows_with_candidates: 0,
        rows_without_candidates: 1,
        rows_with_multiple_candidates: 0,
        official_ready_candidate_count: 0,
        benchmark_ready_candidate_count: 0,
        diagnostic_candidate_count: 0,
        candidate_status: RowCandleCandidateReportStatus::NoCandidates,
        reason_codes: Vec::new(),
    };
    let no_gaps = build_gap_expansion_consistency_report(
        &[sample_gap_map(OfficialCandleGapStatus::NoGapsDetected)],
        &[sample_expansion(
            "AAPL",
            OfficialCandleExpansionFinalStatus::StillMissingOfficialCandles,
            1,
        )],
        &candidate_report,
    );
    assert_eq!(
        no_gaps.consistency_status,
        GapExpansionConsistencyStatus::GapMapSaysNoGapsButClosureHasRemainingGaps
    );

    let wrong_job = build_gap_expansion_consistency_report(
        &[sample_gap_map(
            OfficialCandleGapStatus::MissingOfficialCandles,
        )],
        &[sample_expansion(
            "MSFT",
            OfficialCandleExpansionFinalStatus::CandleCoverageExpanded,
            1,
        )],
        &candidate_report,
    );
    assert!(matches!(
        wrong_job.consistency_status,
        GapExpansionConsistencyStatus::ExpansionAddedSeriesButNoMatches
            | GapExpansionConsistencyStatus::AcquisitionJobDidNotTargetGap
            | GapExpansionConsistencyStatus::AddedSeriesDoesNotMatchGapKey
    ));
}

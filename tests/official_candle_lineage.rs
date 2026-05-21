use soma_zero::{
    ComparableEvidenceSourceClass, OfficialCandleCoverageGapMap, OfficialCandleExpansionReport,
    OfficialCandleLineageTerminalStatus, ProviderMarket, RowCandleCandidate,
    RowCandleCandidateBucket, RowCandleCandidateReport, RowCandleCandidateReportStatus,
    RowCandleCandidateStatus, TimeframeAlignmentStatus, TimestampAlignmentV2Status,
    build_official_candle_lineage_report,
};

fn row(status: RowCandleCandidateStatus) -> soma_zero::ComparableCommitteeEvidenceRow {
    soma_zero::ComparableCommitteeEvidenceRow {
        row_id: format!("row-{status:?}"),
        symbol: "AAPL".to_string(),
        market: ProviderMarket::USEquity,
        timeframe: "1d".to_string(),
        horizon_bars: 3,
        timestamp_ms: 1_700_000_000_000,
        source_kind: "OfficialNonCrypto".to_string(),
        source_class: ComparableEvidenceSourceClass::OfficialNonCrypto,
        scenario_row_id: Some(format!("row-{status:?}")),
        committee_decision_id: None,
        committee_final_action: "Approve".to_string(),
        chair_decision: None,
        risk_governor_decision: Some("Approve".to_string()),
        baseline_action: Some("Approve".to_string()),
        external_action: None,
        no_trade_baseline_action: "NoTrade".to_string(),
        outcome_label: None,
        net_return_pct: None,
        cost_bps: 0.0,
        slippage_bps: 0.0,
        committee_vs_baseline_delta: None,
        committee_vs_notrade_delta: None,
        risk_denied_value_proxy: None,
        no_trade_value_proxy: None,
        outcome_reference_available: false,
        baseline_reference_available: true,
        no_trade_counterfactual_available: true,
        risk_denied_counterfactual_available: true,
        external_reference_available: false,
        row_level: true,
        summary_derived: false,
        no_lookahead_safe: true,
        official_readiness_eligible: true,
        diagnostic_only: false,
        candle_coverage_available: status == RowCandleCandidateStatus::CandidateFound,
        matched_candle_series_id: None,
        candle_match_status: None,
        candle_official_ready_match: false,
        candle_benchmark_ready_match: false,
        candle_diagnostic_only: false,
        reason_codes: Vec::new(),
    }
}

fn candidate_report_for(status: RowCandleCandidateStatus) -> RowCandleCandidateReport {
    let candidate = RowCandleCandidate {
        row_id: "row".to_string(),
        candle_series_id: "series".to_string(),
        candidate_score: 100,
        source_class: ComparableEvidenceSourceClass::OfficialNonCrypto,
        symbol_match: true,
        market_match: true,
        venue_match: true,
        timeframe_match: status != RowCandleCandidateStatus::TimeframeMismatch,
        timestamp_range_match: status != RowCandleCandidateStatus::TimestampOutsideRange,
        future_window_available: status != RowCandleCandidateStatus::MissingFutureWindow,
        official_ready_possible: status == RowCandleCandidateStatus::CandidateFound,
        benchmark_ready_possible: status == RowCandleCandidateStatus::CandidateFound,
        diagnostic_only: status == RowCandleCandidateStatus::DiagnosticOnly,
        timeframe_alignment_status: if status == RowCandleCandidateStatus::TimeframeMismatch {
            TimeframeAlignmentStatus::IncompatibleUpsample
        } else {
            TimeframeAlignmentStatus::ExactMatch
        },
        timestamp_alignment_status: if status == RowCandleCandidateStatus::TimestampOutsideRange {
            TimestampAlignmentV2Status::OutsideCandleRange
        } else {
            TimestampAlignmentV2Status::ExactMatch
        },
        matched_candle_timestamp_ms: Some(1_700_000_000_000),
        reason_codes: if status == RowCandleCandidateStatus::DiagnosticOnly {
            vec![soma_zero::ReasonCode::RejectedNoLookaheadReference]
        } else {
            Vec::new()
        },
    };
    RowCandleCandidateReport {
        candidates_by_row: vec![RowCandleCandidateBucket {
            row_id: format!("row-{status:?}"),
            normalized_key: soma_zero::NormalizedMatchKey {
                market: ProviderMarket::USEquity,
                venue: None,
                provider_kind: None,
                provider_symbol: Some("AAPL".to_string()),
                raw_symbol: "AAPL".to_string(),
                normalized_symbol: "AAPL".to_string(),
                timeframe: "1d".to_string(),
                normalized_timeframe: "1d".to_string(),
                horizon_bars: 3,
                timestamp_ms: 1_700_000_000_000,
                timestamp_policy: soma_zero::TimestampPolicyKind::ExactEpochMs,
                adjusted_price_policy: None,
                source_class: ComparableEvidenceSourceClass::OfficialNonCrypto,
                reason_codes: Vec::new(),
            },
            status,
            selected_candle_series_id: Some("series".to_string()),
            candidates: vec![candidate],
            reason_codes: Vec::new(),
        }],
        rows_with_candidates: 1,
        rows_without_candidates: 0,
        rows_with_multiple_candidates: 0,
        official_ready_candidate_count: usize::from(
            status == RowCandleCandidateStatus::CandidateFound,
        ),
        benchmark_ready_candidate_count: usize::from(
            status == RowCandleCandidateStatus::CandidateFound,
        ),
        diagnostic_candidate_count: usize::from(status == RowCandleCandidateStatus::DiagnosticOnly),
        candidate_status: RowCandleCandidateReportStatus::HealthyCandidates,
        reason_codes: Vec::new(),
    }
}

#[test]
fn official_candle_lineage_maps_terminal_states_deterministically() {
    let statuses = [
        (
            RowCandleCandidateStatus::CandidateFound,
            OfficialCandleLineageTerminalStatus::BackfillClosed,
        ),
        (
            RowCandleCandidateStatus::SymbolMismatch,
            OfficialCandleLineageTerminalStatus::BlockedSymbolMismatch,
        ),
        (
            RowCandleCandidateStatus::TimeframeMismatch,
            OfficialCandleLineageTerminalStatus::BlockedTimeframeMismatch,
        ),
        (
            RowCandleCandidateStatus::TimestampOutsideRange,
            OfficialCandleLineageTerminalStatus::BlockedTimestampMismatch,
        ),
        (
            RowCandleCandidateStatus::MissingFutureWindow,
            OfficialCandleLineageTerminalStatus::BlockedMissingFutureWindow,
        ),
        (
            RowCandleCandidateStatus::MissingProvenance,
            OfficialCandleLineageTerminalStatus::BlockedMissingProvenance,
        ),
        (
            RowCandleCandidateStatus::MissingPreflight,
            OfficialCandleLineageTerminalStatus::BlockedMissingPreflight,
        ),
        (
            RowCandleCandidateStatus::SourceIneligible,
            OfficialCandleLineageTerminalStatus::BlockedSourceIneligible,
        ),
    ];
    for (status, expected) in statuses {
        let report = build_official_candle_lineage_report(
            &[row(status)],
            &candidate_report_for(status),
            &Vec::<OfficialCandleCoverageGapMap>::new(),
            &Vec::<OfficialCandleExpansionReport>::new(),
            &[],
            &[],
            &[],
        );
        assert_eq!(report.traces[0].terminal_status, expected);
    }
}

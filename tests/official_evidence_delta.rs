use soma_zero::{
    BenchmarkStorageAudit, CalibrationSummary, CoreCheckGateResult,
    CoreCheckedBenchmarkRecommendation, CoreCheckedBenchmarkReport, CoreCheckedBenchmarkStatus,
    ModelComparisonSummary, ModelUsefulnessGateResult, OfficialEvidenceDelta, PerformanceSummary,
    ReasonCode, RiskAiInteractionReport, SelectedOfficialDatasets, VenueCoverageExpansionReport,
    VenueCoverageStatus, VenueCoverageTargetResult, VenueGroup, build_official_evidence_delta,
};

fn core_report(
    ready_datasets: usize,
    outcomes: usize,
    venues: &[VenueGroup],
    brier: f64,
    drawdown_delta: f64,
    denial_rate: f64,
) -> CoreCheckedBenchmarkReport {
    CoreCheckedBenchmarkReport {
        benchmark_id: "delta".to_string(),
        core_check_gate: CoreCheckGateResult {
            core_check_ran: true,
            core_status: None,
            passed: true,
            failed_reasons: vec![],
            warnings: vec![],
            reason_codes: vec![],
        },
        dataset_selection: Some(SelectedOfficialDatasets {
            selected_entries: (0..ready_datasets)
                .map(|index| format!("dataset-{index}"))
                .collect(),
            skipped_entries: vec![],
            crypto_entries: if venues.contains(&VenueGroup::Crypto) {
                vec!["crypto".to_string()]
            } else {
                vec![]
            },
            korean_equity_entries: if venues.contains(&VenueGroup::KoreanEquity) {
                vec!["krx".to_string()]
            } else {
                vec![]
            },
            us_equity_entries: if venues.contains(&VenueGroup::USEquity) {
                vec!["us".to_string()]
            } else {
                vec![]
            },
            missing_auth_entries: vec![],
            failed_preflight_entries: vec![],
            coverage_status: soma_zero::OfficialDatasetCoverageStatus::MultiVenue,
            reason_codes: vec![],
        }),
        dataset_bundle: Some(soma_zero::OfficialBenchmarkDatasetBundle {
            dataset_paths: vec![],
            feature_schema: soma_zero::FeatureSchema::from_feature_names(&[]),
            dataset_row_counts: Default::default(),
            label_counts: [("Win".to_string(), outcomes)].into_iter().collect(),
            split_counts: Default::default(),
            fold_counts: Default::default(),
            no_lookahead_report: "safe".to_string(),
            storage_bytes: 0,
            reason_codes: vec![],
        }),
        baseline_report: Some(PerformanceSummary::default()),
        external_report: None,
        calibration_report: Some(CalibrationSummary {
            total_count: outcomes,
            avg_brier_score: brier,
            avg_expected_calibration_error: 0.01,
            acceptable: true,
        }),
        model_comparison_report: Some(ModelComparisonSummary {
            compared_datasets: ready_datasets,
            external_better_count: 0,
            avg_delta_net_return_pct: 0.0,
            avg_delta_max_drawdown_pct: drawdown_delta,
            avg_delta_profit_factor: 0.0,
        }),
        risk_ai_interaction_report: Some(RiskAiInteractionReport {
            model_id: "delta".to_string(),
            total_signals: 10,
            approved_candidates: 10,
            denied_by_risk: 0,
            no_trade_by_signal: 0,
            no_trade_by_risk: 0,
            emergency_stop_count: 0,
            cooldown_count: 0,
            avoided_loss_count: 0,
            missed_gain_count: 0,
            defensive_value: 0.0,
            opportunity_cost: 0.0,
            denial_rate,
            approval_rate: 1.0 - denial_rate,
            reason_code_counts: vec![],
            warnings: vec![],
            reason_codes: vec![],
        }),
        storage_audit: BenchmarkStorageAudit {
            collection_bytes: 0,
            dataset_export_bytes: 0,
            prediction_bytes: 0,
            report_bytes: 0,
            raw_archive_bytes: 0,
            canonical_bytes: 0,
            budget_exceeded: false,
            largest_files: vec![],
            retention_actions: vec![],
            reason_codes: vec![],
        },
        usefulness_gate_result: ModelUsefulnessGateResult {
            passed: true,
            failed_gates: vec![],
            warnings: vec![],
            reason_codes: vec![],
        },
        final_status: CoreCheckedBenchmarkStatus::BaselineOnlyEvaluated,
        next_recommendation: CoreCheckedBenchmarkRecommendation::HoldCurrentScope,
        blockers: vec![],
        warnings: vec![],
        reason_codes: vec![ReasonCode::OfficialAiBenchmarkRan],
    }
}

fn coverage(venues: &[VenueGroup]) -> VenueCoverageExpansionReport {
    VenueCoverageExpansionReport {
        plan_id: "coverage".to_string(),
        target_results: venues
            .iter()
            .map(|venue| VenueCoverageTargetResult {
                venue_group: *venue,
                ready_datasets: 1,
                outcome_records: 20,
                symbol_count: 1,
                timeframe_count: 1,
                auth_blocked: false,
                passed: true,
                warnings: vec![],
                reason_codes: vec![],
            })
            .collect(),
        crypto_status: "ok".to_string(),
        korean_equity_status: "ok".to_string(),
        us_equity_status: "ok".to_string(),
        missing_auth_summary: vec![],
        skipped_summary: vec![],
        coverage_status: VenueCoverageStatus::MultiVenueReady,
        warnings: vec![],
        reason_codes: vec![],
    }
}

#[test]
fn missing_previous_report_is_not_comparable() {
    let delta = build_official_evidence_delta(
        None,
        Some(&core_report(1, 20, &[VenueGroup::Crypto], 0.1, 0.0, 0.1)),
        &coverage(&[VenueGroup::Crypto]),
    );
    assert!(!delta.comparable);
}

#[test]
fn added_ready_datasets_and_outcomes_are_counted() {
    let previous = core_report(1, 20, &[VenueGroup::Crypto], 0.1, 0.0, 0.1);
    let current = core_report(
        3,
        60,
        &[VenueGroup::Crypto, VenueGroup::USEquity],
        0.1,
        0.0,
        0.1,
    );
    let delta = build_official_evidence_delta(
        Some(&previous),
        Some(&current),
        &coverage(&[VenueGroup::Crypto, VenueGroup::USEquity]),
    );

    assert_eq!(delta.added_ready_datasets, 2);
    assert_eq!(delta.added_outcome_records, 40);
    assert!(delta.added_venues.contains(&"USEquity".to_string()));
}

#[test]
fn regression_is_detected_when_calibration_drawdown_or_risk_worsens() {
    let previous = core_report(1, 20, &[VenueGroup::Crypto], 0.05, 0.01, 0.1);
    let current = core_report(1, 20, &[VenueGroup::Crypto], 0.20, 0.10, 0.3);
    let delta = build_official_evidence_delta(
        Some(&previous),
        Some(&current),
        &coverage(&[VenueGroup::Crypto]),
    );

    assert!(!delta.regressions.is_empty());
}

#[test]
fn delta_report_is_deterministic() {
    let previous = core_report(1, 20, &[VenueGroup::Crypto], 0.05, 0.01, 0.1);
    let current = core_report(
        2,
        40,
        &[VenueGroup::Crypto, VenueGroup::USEquity],
        0.05,
        0.01,
        0.1,
    );
    let coverage = coverage(&[VenueGroup::Crypto, VenueGroup::USEquity]);

    let first: OfficialEvidenceDelta =
        build_official_evidence_delta(Some(&previous), Some(&current), &coverage);
    let second: OfficialEvidenceDelta =
        build_official_evidence_delta(Some(&previous), Some(&current), &coverage);

    assert_eq!(first, second);
}

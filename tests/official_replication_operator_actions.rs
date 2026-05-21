mod common;

use soma_zero::{
    DataFreshnessTier, OfficialCandleCoverageReport, OfficialCandleCoverageStatus,
    OfficialEvidenceReplicationConfig, OfficialProviderReadinessReport,
    OfficialProviderReadinessStatus, OfficialReplicationArtifactInventory,
    OfficialReplicationOperatorActionPlanner, OfficialSufficiencyReplicationReport,
    OfficialSufficiencyReplicationStatus, ProviderCostTier, ProviderDataSubject,
    ProviderEntitlementStatus, ProviderEntitlementStatusKind, ProviderKind, ProviderRealityReport,
    ProviderRealitySummary, ReasonCode, build_default_provider_catalog,
    default_provider_cost_profiles, default_provider_freshness_profiles,
};

fn write_readiness_report(name: &str) -> std::path::PathBuf {
    let dir = common::output_dir(&format!("{name}-readiness"));
    let report = OfficialProviderReadinessReport {
        report_id: name.to_string(),
        catalog: build_default_provider_catalog(),
        credential_statuses: Vec::new(),
        selection_results: Vec::new(),
        implemented_providers: vec!["alphavantage".to_string()],
        missing_auth_actions: vec![
            "krx missing auth".to_string(),
            "alphavantage missing auth".to_string(),
            "data-go-kr missing auth".to_string(),
        ],
        deferred_provider_actions: Vec::new(),
        official_ready_markets: Vec::new(),
        research_only_markets: Vec::new(),
        final_status: OfficialProviderReadinessStatus::MissingProviderEndpointProfile,
        reason_codes: vec![ReasonCode::ProviderReadinessReportBuilt],
    };
    report.write_to_dir(&dir).expect("write readiness")
}

fn write_reality_report(name: &str) -> std::path::PathBuf {
    let dir = common::output_dir(&format!("{name}-reality"));
    let report = ProviderRealityReport {
        report_id: name.to_string(),
        freshness_profiles: default_provider_freshness_profiles(),
        cost_profiles: default_provider_cost_profiles(),
        entitlement_statuses: vec![
            ProviderEntitlementStatus {
                provider_subject: ProviderDataSubject::Provider(ProviderKind::KrxOpenApi),
                freshness_available: vec![DataFreshnessTier::Eod],
                cost_tier: ProviderCostTier::Free,
                auth_ready: true,
                approval_ready: false,
                endpoint_template_ready: true,
                realtime_entitlement_ready: false,
                delayed_entitlement_ready: false,
                official_readiness_eligible: false,
                research_only: false,
                status: ProviderEntitlementStatusKind::MissingApproval,
                reason_codes: vec![ReasonCode::MissingApproval],
            },
            ProviderEntitlementStatus {
                provider_subject: ProviderDataSubject::Provider(ProviderKind::Alpaca),
                freshness_available: vec![DataFreshnessTier::RealtimeIex],
                cost_tier: ProviderCostTier::Free,
                auth_ready: false,
                approval_ready: false,
                endpoint_template_ready: true,
                realtime_entitlement_ready: false,
                delayed_entitlement_ready: false,
                official_readiness_eligible: false,
                research_only: true,
                status: ProviderEntitlementStatusKind::MissingAuth,
                reason_codes: vec![ReasonCode::MissingAuth],
            },
        ],
        compatibility_results: Vec::new(),
        recommendations: Vec::new(),
        operator_actions: Vec::new(),
        final_summary: vec![ProviderRealitySummary::KRXApprovalPending],
        reason_codes: vec![ReasonCode::ProviderRealityReportBuilt],
    };
    report.write_to_dir(&dir).expect("write reality")
}

#[test]
fn operator_actions_cover_missing_artifacts_and_are_sorted_without_secrets() {
    let inventory = OfficialReplicationArtifactInventory::from_paths(&[]);
    let plan = OfficialReplicationOperatorActionPlanner::default().build(
        &OfficialEvidenceReplicationConfig::default(),
        &inventory,
        Some(&OfficialCandleCoverageReport {
            official_rows: 0,
            rows_with_candles: 0,
            rows_with_future_window: 0,
            rows_no_lookahead_safe: 0,
            missing_candle_rows: 0,
            missing_future_window_rows: 0,
            timestamp_mismatch_rows: 0,
            symbol_mismatch_rows: 0,
            timeframe_mismatch_rows: 0,
            gap_rows: 0,
            duplicate_timestamp_rows: 0,
            coverage_status: OfficialCandleCoverageStatus::MissingOfficialCandles,
            reason_codes: vec![],
        }),
        None,
    );
    let ids = plan
        .actions
        .iter()
        .map(|action| action.action_id.as_str())
        .collect::<Vec<_>>();
    assert!(ids.windows(2).all(|pair| pair[0] <= pair[1]));
    assert!(ids.contains(&"RunProviderReadiness"));
    assert!(ids.contains(&"RunProviderReality"));
    assert!(ids.contains(&"RunOfficialAcquire"));
    assert!(ids.contains(&"ProvideOfficialCandleSeries"));
    assert!(ids.contains(&"RunCommitteeBuildReferences"));
    assert!(plan.blockers.iter().any(|line| line.contains("official")));
    assert!(plan.actions.iter().all(|action| action.safe_to_run));
    assert!(
        plan.actions
            .iter()
            .flat_map(|action| action.env_var_names.iter())
            .all(|name| !name.contains('=') && name == &name.to_ascii_uppercase())
    );
}

#[test]
fn operator_actions_include_provider_auth_and_benchmark_steps() {
    let readiness = write_readiness_report("operator-actions");
    let reality = write_reality_report("operator-actions");
    let inventory = OfficialReplicationArtifactInventory::from_paths(&vec![
        readiness.display().to_string(),
        reality.display().to_string(),
    ]);
    let sufficiency = OfficialSufficiencyReplicationReport {
        previous_controlled_status: None,
        current_official_status: OfficialSufficiencyReplicationStatus::OfficialSufficiencyPassed,
        official_row_count: 1,
        non_crypto_official_row_count: 1,
        official_reference_count: 1,
        outcome_link_count: 1,
        baseline_reference_count: 1,
        no_trade_counterfactual_count: 1,
        risk_denied_counterfactual_count: 1,
        summary_derived_ratio: 0.0,
        diagnostic_only_ratio: 0.0,
        controlled_only_ratio: 0.0,
        crypto_only_ratio: 0.0,
        passed_for_controlled: true,
        passed_for_official: true,
        remaining_gaps: Vec::new(),
        final_status: OfficialSufficiencyReplicationStatus::OfficialSufficiencyPassed,
        reason_codes: Vec::new(),
    };
    let plan = OfficialReplicationOperatorActionPlanner::default().build(
        &OfficialEvidenceReplicationConfig::default(),
        &inventory,
        Some(&OfficialCandleCoverageReport {
            official_rows: 1,
            rows_with_candles: 1,
            rows_with_future_window: 1,
            rows_no_lookahead_safe: 1,
            missing_candle_rows: 0,
            missing_future_window_rows: 0,
            timestamp_mismatch_rows: 0,
            symbol_mismatch_rows: 0,
            timeframe_mismatch_rows: 0,
            gap_rows: 0,
            duplicate_timestamp_rows: 0,
            coverage_status: OfficialCandleCoverageStatus::HealthyOfficialCandleCoverage,
            reason_codes: vec![],
        }),
        Some(&sufficiency),
    );
    let ids = plan
        .actions
        .iter()
        .map(|action| action.action_id.as_str())
        .collect::<Vec<_>>();
    assert!(ids.contains(&"SetKrxApiKey"));
    assert!(ids.contains(&"SetAlphaVantageApiKey"));
    assert!(ids.contains(&"SetDataGoKrServiceKey"));
    assert!(ids.contains(&"SetKrxEndpointTemplate"));
    assert!(ids.contains(&"WaitForKrxApproval"));
    assert!(ids.contains(&"SetAlpacaKeys"));
    assert!(ids.contains(&"RunCommitteeOfficialBenchmark"));
}

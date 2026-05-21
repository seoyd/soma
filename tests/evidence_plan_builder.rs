use soma_zero::{
    EvidenceLaneKind, EvidenceLaneStatus, EvidencePlanBuilder, ExecutableEvidencePlanConfig,
    ProviderCostProfile, ProviderDataSubject, ProviderEntitlementStatus,
    ProviderEntitlementStatusKind, ProviderFreshnessProfile, ProviderKind, ProviderMarket,
    ProviderRealityReport, provider_cost_profile, provider_freshness_profile,
};

fn entitlement(
    subject: ProviderDataSubject,
    status: ProviderEntitlementStatusKind,
) -> ProviderEntitlementStatus {
    let freshness = provider_freshness_profile(subject);
    let cost = provider_cost_profile(subject);
    ProviderEntitlementStatus {
        provider_subject: subject,
        freshness_available: freshness.available_freshness_tiers,
        cost_tier: cost.cost_tier,
        auth_ready: !matches!(status, ProviderEntitlementStatusKind::MissingAuth),
        approval_ready: !matches!(status, ProviderEntitlementStatusKind::MissingApproval),
        endpoint_template_ready: !matches!(
            status,
            ProviderEntitlementStatusKind::MissingEndpointTemplate
        ),
        realtime_entitlement_ready: matches!(
            status,
            ProviderEntitlementStatusKind::ReadyForRealtimeResearch
                | ProviderEntitlementStatusKind::ReadyForRealtimeResearchIexOnly
                | ProviderEntitlementStatusKind::ReadyForCryptoResearch
        ),
        delayed_entitlement_ready: matches!(
            status,
            ProviderEntitlementStatusKind::ReadyForIntradayResearch
                | ProviderEntitlementStatusKind::ReadyForRealtimeResearch
                | ProviderEntitlementStatusKind::ReadyForRealtimeResearchIexOnly
                | ProviderEntitlementStatusKind::ReadyForCryptoResearch
        ),
        official_readiness_eligible: !matches!(subject, ProviderDataSubject::YFinanceResearch),
        research_only: matches!(subject, ProviderDataSubject::YFinanceResearch),
        status,
        reason_codes: vec![],
    }
}

fn report(entitlement_statuses: Vec<ProviderEntitlementStatus>) -> ProviderRealityReport {
    ProviderRealityReport {
        report_id: "builder-test".to_string(),
        freshness_profiles: Vec::<ProviderFreshnessProfile>::new(),
        cost_profiles: Vec::<ProviderCostProfile>::new(),
        entitlement_statuses,
        compatibility_results: vec![],
        recommendations: vec![],
        operator_actions: vec![],
        final_summary: vec![],
        reason_codes: vec![],
    }
}

#[test]
fn missing_krx_approval_creates_skipped_lane_and_operator_action() {
    let plan = EvidencePlanBuilder::default()
        .from_provider_reality(
            &report(vec![
                entitlement(
                    ProviderDataSubject::Provider(ProviderKind::KrxOpenApi),
                    ProviderEntitlementStatusKind::MissingApproval,
                ),
                entitlement(
                    ProviderDataSubject::Provider(ProviderKind::Upbit),
                    ProviderEntitlementStatusKind::ReadyForCryptoResearch,
                ),
            ]),
            &ExecutableEvidencePlanConfig {
                output_root: "target/sprint10-tests/builder-krx".to_string(),
                allow_yfinance_research: false,
                ..ExecutableEvidencePlanConfig::default()
            },
        )
        .expect("plan");

    assert!(
        plan.skipped_lanes
            .iter()
            .any(|lane| lane.provider_kind == Some(ProviderKind::KrxOpenApi)
                && lane.lane_kind == EvidenceLaneKind::KoreanEquityEodEvidence
                && lane.lane_status == EvidenceLaneStatus::SkippedMissingApproval)
    );
    assert!(
        plan.operator_actions
            .iter()
            .any(|action| action == "WaitForKrxApproval")
    );
    assert!(
        plan.runnable_lanes
            .iter()
            .any(|lane| lane.provider_kind == Some(ProviderKind::Upbit)
                && lane.market == ProviderMarket::Crypto)
    );
}

#[test]
fn alphavantage_missing_auth_creates_skipped_us_eod_lane() {
    let plan = EvidencePlanBuilder::default()
        .from_provider_reality(
            &report(vec![entitlement(
                ProviderDataSubject::Provider(ProviderKind::AlphaVantage),
                ProviderEntitlementStatusKind::MissingAuth,
            )]),
            &ExecutableEvidencePlanConfig {
                output_root: "target/sprint10-tests/builder-alpha".to_string(),
                allow_yfinance_research: false,
                ..ExecutableEvidencePlanConfig::default()
            },
        )
        .expect("plan");

    assert!(plan.skipped_lanes.iter().any(|lane| lane.provider_kind
        == Some(ProviderKind::AlphaVantage)
        && lane.lane_kind == EvidenceLaneKind::USEquityEodEvidence
        && lane.lane_status == EvidenceLaneStatus::SkippedMissingAuth));
    assert!(
        plan.operator_actions
            .iter()
            .any(|action| action == "SetAlphaVantageAuth")
    );
}

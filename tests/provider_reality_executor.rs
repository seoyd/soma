mod common;

use std::fs;

use soma_zero::{
    EvidenceLaneKind, ExecutableEvidencePlanConfig, ExplicitEvidenceLaneConfig,
    ProviderCostProfile, ProviderDataSubject, ProviderEntitlementStatus,
    ProviderEntitlementStatusKind, ProviderFreshnessProfile, ProviderKind,
    ProviderRealityEvidenceExecutor, ProviderRealityEvidenceFinalStatus, ProviderRealityReport,
    provider_cost_profile, provider_freshness_profile,
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

fn report_path(name: &str, statuses: Vec<ProviderEntitlementStatus>) -> String {
    let dir = common::output_dir(name);
    let path = dir.join("provider_reality_report.json");
    let report = ProviderRealityReport {
        report_id: name.to_string(),
        freshness_profiles: Vec::<ProviderFreshnessProfile>::new(),
        cost_profiles: Vec::<ProviderCostProfile>::new(),
        entitlement_statuses: statuses,
        compatibility_results: vec![],
        recommendations: vec![],
        operator_actions: vec![],
        final_summary: vec![],
        reason_codes: vec![],
    };
    fs::write(&path, report.to_json_string().expect("json")).expect("write");
    path.display().to_string()
}

#[test]
fn executor_reports_no_runnable_lanes_when_plan_is_empty() {
    let report = ProviderRealityEvidenceExecutor::default()
        .run(&ExecutableEvidencePlanConfig {
            output_root: common::output_dir("executor-empty").display().to_string(),
            allow_yfinance_research: false,
            explicit_lanes: vec![],
            ..ExecutableEvidencePlanConfig::default()
        })
        .expect("report");
    assert_eq!(
        report.final_status,
        ProviderRealityEvidenceFinalStatus::NoRunnableLanes
    );
}

#[test]
fn executor_reports_crypto_only_ran_for_upbit_lane() {
    let report = ProviderRealityEvidenceExecutor::default()
        .run(&ExecutableEvidencePlanConfig {
            output_root: common::output_dir("executor-upbit").display().to_string(),
            allow_yfinance_research: false,
            explicit_lanes: vec![ExplicitEvidenceLaneConfig {
                lane_kind: EvidenceLaneKind::CryptoIntradayEvidence,
                provider: "upbit".to_string(),
                symbols: vec!["BTC-KRW".to_string()],
                enabled: true,
                output_subdir: None,
                max_rows: None,
                max_requests: None,
                allow_full_history: false,
                allow_all_symbols: false,
                reason_codes: vec![],
            }],
            ..ExecutableEvidencePlanConfig::default()
        })
        .expect("report");
    assert_eq!(
        report.final_status,
        ProviderRealityEvidenceFinalStatus::CryptoOnlyRan
    );
}

#[test]
fn executor_reports_research_only_for_yfinance_lane() {
    let report = ProviderRealityEvidenceExecutor::default()
        .run(&ExecutableEvidencePlanConfig {
            output_root: common::output_dir("executor-yfinance")
                .display()
                .to_string(),
            explicit_lanes: vec![ExplicitEvidenceLaneConfig {
                lane_kind: EvidenceLaneKind::YFinanceResearchFallback,
                provider: "yfinance".to_string(),
                symbols: vec!["AAPL".to_string()],
                enabled: true,
                output_subdir: None,
                max_rows: None,
                max_requests: None,
                allow_full_history: false,
                allow_all_symbols: false,
                reason_codes: vec![],
            }],
            ..ExecutableEvidencePlanConfig::default()
        })
        .expect("report");
    assert_eq!(
        report.final_status,
        ProviderRealityEvidenceFinalStatus::ResearchOnlyYFinanceRan
    );
}

#[test]
fn executor_reports_eod_evidence_ran_for_ready_alpha_lane() {
    let report = ProviderRealityEvidenceExecutor::default()
        .run(&ExecutableEvidencePlanConfig {
            output_root: common::output_dir("executor-alpha-ready")
                .display()
                .to_string(),
            allow_yfinance_research: false,
            provider_reality_report_path: Some(report_path(
                "executor-alpha-ready-report",
                vec![entitlement(
                    ProviderDataSubject::Provider(ProviderKind::AlphaVantage),
                    ProviderEntitlementStatusKind::ReadyForEodResearch,
                )],
            )),
            ..ExecutableEvidencePlanConfig::default()
        })
        .expect("report");
    assert_eq!(
        report.final_status,
        ProviderRealityEvidenceFinalStatus::EodEvidenceRan
    );
}

#[test]
fn executor_reports_multivenue_when_two_official_lanes_run() {
    let report = ProviderRealityEvidenceExecutor::default()
        .run(&ExecutableEvidencePlanConfig {
            output_root: common::output_dir("executor-multi").display().to_string(),
            allow_yfinance_research: false,
            provider_reality_report_path: Some(report_path(
                "executor-multi-report",
                vec![
                    entitlement(
                        ProviderDataSubject::Provider(ProviderKind::AlphaVantage),
                        ProviderEntitlementStatusKind::ReadyForEodResearch,
                    ),
                    entitlement(
                        ProviderDataSubject::Provider(ProviderKind::Upbit),
                        ProviderEntitlementStatusKind::ReadyForCryptoResearch,
                    ),
                ],
            )),
            ..ExecutableEvidencePlanConfig::default()
        })
        .expect("report");
    assert_eq!(
        report.final_status,
        ProviderRealityEvidenceFinalStatus::MultiVenueEvidenceRan
    );
}

#[test]
fn executor_reports_budget_blocked_when_budget_is_too_small() {
    let report = ProviderRealityEvidenceExecutor::default()
        .run(&ExecutableEvidencePlanConfig {
            output_root: common::output_dir("executor-budget").display().to_string(),
            allow_yfinance_research: false,
            explicit_lanes: vec![ExplicitEvidenceLaneConfig {
                lane_kind: EvidenceLaneKind::CryptoIntradayEvidence,
                provider: "upbit".to_string(),
                symbols: vec!["BTC-KRW".to_string()],
                enabled: true,
                output_subdir: None,
                max_rows: Some(500),
                max_requests: Some(10),
                allow_full_history: false,
                allow_all_symbols: false,
                reason_codes: vec![],
            }],
            max_total_bytes: 100,
            ..ExecutableEvidencePlanConfig::default()
        })
        .expect("report");
    assert_eq!(
        report.final_status,
        ProviderRealityEvidenceFinalStatus::BudgetBlocked
    );
}

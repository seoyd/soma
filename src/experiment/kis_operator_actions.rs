use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_reason_codes};
use crate::experiment::OperatorActionPriority;

use super::kis_auth_readiness::{
    KIS_APP_KEY_ENV_VAR, KIS_APP_SECRET_ENV_VAR, KIS_BASE_URL_ENV_VAR, KIS_WS_APPROVAL_KEY_ENV_VAR,
    KISAuthReadinessReport, KISAuthReadinessStatus,
};
use super::kis_canonical_batch_validation::KISCanonicalValidationReport;
use super::kis_endpoint_policy::KISEndpointPolicyReport;
use super::kis_market_data_activation::KISMarketDataActivationConfig;
use super::kis_symbol_whitelist::KISSymbolWhitelist;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum KISOperatorActionKind {
    SetKISAppKey,
    SetKISAppSecret,
    SetKISBaseUrl,
    SetKISWebSocketApprovalKey,
    RunProviderReadiness,
    RunProviderReality,
    RunKISMarketDataDryRun,
    RunKISMarketDataCollect,
    ProvideKISCanonicalCsv,
    ProvideKISProvenance,
    RunKISPreflight,
    RunOfficialReplication,
    RunCandlePack,
    RunCandleSufficiency,
    RunOutcomeLinkClosure,
    RunCompleteRowCloseV2,
    RunOfficialEvidenceScaleout,
    RunOfficialEvidenceDiversitySweep,
    RunCommitteeOfficialBenchmark,
    RunCorePerformance,
    KeepKRXAsReference,
    ReduceScope,
    RemoveUnsafeBrokerEndpoint,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct KISOperatorAction {
    pub action_id: String,
    pub action_kind: KISOperatorActionKind,
    pub priority: OperatorActionPriority,
    #[serde(default)]
    pub env_var_names: Vec<String>,
    pub description: String,
    #[serde(default)]
    pub command_suggestion: Option<String>,
    #[serde(default)]
    pub expected_output_artifact: Option<String>,
    pub safe_to_run: bool,
    pub reason_codes: Vec<ReasonCode>,
}

pub fn build_kis_operator_actions(
    config: &KISMarketDataActivationConfig,
    auth: &KISAuthReadinessReport,
    endpoint_report: &KISEndpointPolicyReport,
    whitelist: &KISSymbolWhitelist,
    validations: &[KISCanonicalValidationReport],
    budget_exceeded: bool,
) -> Vec<KISOperatorAction> {
    let mut actions = Vec::new();
    if matches!(
        auth.readiness_status,
        KISAuthReadinessStatus::MissingAppKey | KISAuthReadinessStatus::MissingAppKeyAndSecret
    ) {
        actions.push(action(
            "set-kis-app-key",
            KISOperatorActionKind::SetKISAppKey,
            OperatorActionPriority::Required,
            vec![KIS_APP_KEY_ENV_VAR.to_string()],
            "Set KIS_APP_KEY locally to enable bounded REST market-data collection.",
            Some("cargo run --quiet --bin soma_experiment -- kis-auth-readiness --config examples/soma_kis_auth_readiness.toml".to_string()),
            None,
            vec![ReasonCode::KISOperatorActionPlanBuilt, ReasonCode::MissingApiKey],
        ));
    }
    if matches!(
        auth.readiness_status,
        KISAuthReadinessStatus::MissingAppSecret | KISAuthReadinessStatus::MissingAppKeyAndSecret
    ) {
        actions.push(action(
            "set-kis-app-secret",
            KISOperatorActionKind::SetKISAppSecret,
            OperatorActionPriority::Required,
            vec![KIS_APP_SECRET_ENV_VAR.to_string()],
            "Set KIS_APP_SECRET locally; never print or persist the secret value.",
            Some("cargo run --quiet --bin soma_experiment -- kis-auth-readiness --config examples/soma_kis_auth_readiness.toml".to_string()),
            None,
            vec![ReasonCode::KISOperatorActionPlanBuilt, ReasonCode::MissingAuth],
        ));
    }
    if matches!(
        auth.readiness_status,
        KISAuthReadinessStatus::MissingBaseUrl
    ) {
        actions.push(action(
            "set-kis-base-url",
            KISOperatorActionKind::SetKISBaseUrl,
            OperatorActionPriority::Required,
            vec![KIS_BASE_URL_ENV_VAR.to_string()],
            "Set KIS_BASE_URL locally using the operator-approved market-data base URL only.",
            Some("cargo run --quiet --bin soma_experiment -- kis-auth-readiness --config examples/soma_kis_auth_readiness.toml".to_string()),
            None,
            vec![ReasonCode::KISOperatorActionPlanBuilt, ReasonCode::MissingEndpointTemplate],
        ));
    }
    if config.run_live_market_data_collection
        && matches!(
            auth.readiness_status,
            KISAuthReadinessStatus::MissingWebSocketApprovalKey
        )
    {
        actions.push(action(
            "set-kis-websocket-approval-key",
            KISOperatorActionKind::SetKISWebSocketApprovalKey,
            OperatorActionPriority::Recommended,
            vec![KIS_WS_APPROVAL_KEY_ENV_VAR.to_string()],
            "Set KIS_WS_APPROVAL_KEY only when realtime quote collection is explicitly enabled.",
            Some("cargo run --quiet --bin soma_experiment -- kis-auth-readiness --config examples/soma_kis_auth_readiness.toml".to_string()),
            None,
            vec![ReasonCode::KISOperatorActionPlanBuilt, ReasonCode::MissingApproval],
        ));
    }
    if endpoint_report.unsafe_endpoint_detected {
        actions.push(action(
            "remove-unsafe-broker-endpoint",
            KISOperatorActionKind::RemoveUnsafeBrokerEndpoint,
            OperatorActionPriority::Required,
            Vec::new(),
            "Remove any broker/order/account KIS endpoint from configs or plans before collection.",
            Some("cargo run --quiet --bin soma_experiment -- kis-endpoint-policy --config examples/soma_kis_endpoint_policy.toml".to_string()),
            Some("kis_endpoint_policy.txt".to_string()),
            vec![ReasonCode::KISOperatorActionPlanBuilt, ReasonCode::KISBrokerEndpointDetected],
        ));
    }
    if whitelist.domestic_count > config.max_domestic_symbols
        || whitelist.overseas_count > config.max_overseas_symbols
        || whitelist.enabled_entries.len()
            > config.max_domestic_symbols + config.max_overseas_symbols
    {
        actions.push(action(
            "reduce-kis-scope",
            KISOperatorActionKind::ReduceScope,
            OperatorActionPriority::Required,
            Vec::new(),
            "Reduce the KIS whitelist to the compact bounded domestic and overseas scope before collection.",
            Some("cargo run --quiet --bin soma_experiment -- kis-symbol-whitelist --config examples/soma_kis_symbol_whitelist_compact.toml".to_string()),
            Some("kis_symbol_whitelist.txt".to_string()),
            vec![ReasonCode::KISOperatorActionPlanBuilt, ReasonCode::DeniedByDefault],
        ));
    }
    if validations.is_empty() && !config.run_fixture_replay && !config.run_local_import {
        actions.push(action(
            "provide-kis-canonical-csv",
            KISOperatorActionKind::ProvideKISCanonicalCsv,
            OperatorActionPriority::Required,
            Vec::new(),
            "Provide bounded local KIS canonical CSV inputs before official-readiness claims.",
            None,
            Some("kis_kr_005930_1d_eod.csv".to_string()),
            vec![
                ReasonCode::KISOperatorActionPlanBuilt,
                ReasonCode::MissingOfficialData,
            ],
        ));
    }
    if validations
        .iter()
        .any(|report| !report.provenance_available)
    {
        actions.push(action(
            "provide-kis-provenance",
            KISOperatorActionKind::ProvideKISProvenance,
            OperatorActionPriority::Required,
            Vec::new(),
            "Provide local KIS provenance JSON for every canonical CSV before official readiness claims.",
            None,
            Some("kis_provenance.json".to_string()),
            vec![ReasonCode::KISOperatorActionPlanBuilt, ReasonCode::MissingOfficialProvenance],
        ));
    }
    if validations.iter().any(|report| !report.preflight_available) {
        actions.push(action(
            "run-kis-preflight",
            KISOperatorActionKind::RunKISPreflight,
            OperatorActionPriority::Required,
            Vec::new(),
            "Provide or regenerate local KIS preflight reports before using official readiness artifacts.",
            Some("cargo run --quiet --bin soma_experiment -- kis-market-data-activate --config examples/soma_kis_market_data_activate_local_import.toml".to_string()),
            Some("kis_preflight.json".to_string()),
            vec![ReasonCode::KISOperatorActionPlanBuilt, ReasonCode::MissingOfficialPreflight],
        ));
    }
    if budget_exceeded {
        actions.push(action(
            "reduce-kis-storage-budget",
            KISOperatorActionKind::ReduceScope,
            OperatorActionPriority::Required,
            Vec::new(),
            "Reduce KIS symbol count or row count to remain within the configured local storage budget.",
            None,
            None,
            vec![ReasonCode::KISOperatorActionPlanBuilt, ReasonCode::BudgetExceeded],
        ));
    }
    if config.run_collection_dry_run {
        actions.push(action(
            "run-kis-market-data-dry-run",
            KISOperatorActionKind::RunKISMarketDataDryRun,
            OperatorActionPriority::Recommended,
            Vec::new(),
            "Run the local research-only KIS dry run before any operator-enabled live collection.",
            Some("cargo run --quiet --bin soma_experiment -- kis-collection-plan --config examples/soma_kis_collection_plan_missing_auth.toml".to_string()),
            Some("kis_collection_batch_plan.txt".to_string()),
            vec![ReasonCode::KISOperatorActionPlanBuilt],
        ));
    }
    if config.run_official_replication {
        actions.push(action(
            "run-kis-official-replication",
            KISOperatorActionKind::RunOfficialReplication,
            OperatorActionPriority::Recommended,
            Vec::new(),
            "Run official replication after KIS canonical, provenance, and preflight artifacts are ready.",
            Some("cargo run --quiet --bin soma_experiment -- kis-market-data-activate --config examples/soma_kis_market_data_activate_local_import.toml".to_string()),
            Some("official_replication_bundle.json".to_string()),
            vec![ReasonCode::KISOperatorActionPlanBuilt],
        ));
    }
    if config.run_candle_sufficiency {
        actions.push(action(
            "run-kis-candle-sufficiency",
            KISOperatorActionKind::RunCandleSufficiency,
            OperatorActionPriority::Recommended,
            Vec::new(),
            "Run the KIS candle sufficiency report before outcome linkage or committee/core reruns.",
            Some("cargo run --quiet --bin soma_experiment -- kis-candle-sufficiency --config examples/soma_kis_candle_sufficiency.toml".to_string()),
            Some("kis_candle_sufficiency.txt".to_string()),
            vec![ReasonCode::KISOperatorActionPlanBuilt],
        ));
    }
    if config.run_outcome_link_closure {
        actions.push(action(
            "run-kis-outcome-link-closure",
            KISOperatorActionKind::RunOutcomeLinkClosure,
            OperatorActionPriority::Recommended,
            Vec::new(),
            "Run the research-only KIS outcome-link closure after candle sufficiency is healthy.",
            Some("cargo run --quiet --bin soma_experiment -- kis-outcome-link-close --config examples/soma_kis_outcome_link_close.toml".to_string()),
            Some("kis_outcome_link_closure.txt".to_string()),
            vec![ReasonCode::KISOperatorActionPlanBuilt],
        ));
    }
    if config.run_official_evidence_diversity_sweep {
        actions.push(action(
            "run-kis-diversity-sweep",
            KISOperatorActionKind::RunOfficialEvidenceDiversitySweep,
            OperatorActionPriority::Recommended,
            Vec::new(),
            "Run the downstream diversity sweep conservatively after KIS official replication artifacts exist.",
            Some("cargo run --quiet --bin soma_experiment -- kis-market-data-activate --config examples/soma_kis_market_data_activate_local_import.toml".to_string()),
            Some("official_evidence_diversity_bundle.json".to_string()),
            vec![ReasonCode::KISOperatorActionPlanBuilt],
        ));
    }
    if config.run_core_performance {
        actions.push(action(
            "run-kis-core-performance",
            KISOperatorActionKind::RunCorePerformance,
            OperatorActionPriority::Recommended,
            Vec::new(),
            "Run the research-only core performance scorecard after KIS outcome-linked artifacts are available.",
            Some("cargo run --quiet --bin soma_experiment -- core-performance --config examples/soma_core_performance_diagnostics_only.toml".to_string()),
            Some("core_performance_scorecard.json".to_string()),
            vec![ReasonCode::KISOperatorActionPlanBuilt],
        ));
    }
    actions.push(action(
        "keep-krx-as-reference",
        KISOperatorActionKind::KeepKRXAsReference,
        OperatorActionPriority::Optional,
        Vec::new(),
        "Retain KRX as an exchange-reference fallback while KIS becomes the primary operational market-data path.",
        Some("cargo run --quiet --bin soma_experiment -- kis-krx-migration --config examples/soma_kis_krx_migration.toml".to_string()),
        Some("kis_krx_migration_report.txt".to_string()),
        vec![ReasonCode::KISOperatorActionPlanBuilt, ReasonCode::KRXRetainedAsReference],
    ));
    actions.sort_by(|left, right| left.action_id.cmp(&right.action_id));
    actions
}

impl KISOperatorAction {
    pub fn to_text(&self) -> String {
        format!(
            "action_id={};action_kind={:?};priority={:?};env_var_names={};description={};command_suggestion={};expected_output_artifact={};safe_to_run={};reason_codes={}",
            self.action_id,
            self.action_kind,
            self.priority,
            self.env_var_names.join("|"),
            self.description,
            self.command_suggestion.clone().unwrap_or_default(),
            self.expected_output_artifact.clone().unwrap_or_default(),
            self.safe_to_run,
            self.reason_codes
                .iter()
                .map(|reason| format!("{reason:?}"))
                .collect::<Vec<_>>()
                .join("|")
        )
    }
}

fn action(
    action_id: &str,
    action_kind: KISOperatorActionKind,
    priority: OperatorActionPriority,
    env_var_names: Vec<String>,
    description: &str,
    command_suggestion: Option<String>,
    expected_output_artifact: Option<String>,
    reason_codes: Vec<ReasonCode>,
) -> KISOperatorAction {
    KISOperatorAction {
        action_id: action_id.to_string(),
        action_kind,
        priority,
        env_var_names,
        description: description.to_string(),
        command_suggestion,
        expected_output_artifact,
        safe_to_run: true,
        reason_codes: stable_reason_codes(&reason_codes),
    }
}

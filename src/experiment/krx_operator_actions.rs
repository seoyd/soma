use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_reason_codes};
use crate::experiment::OperatorActionPriority;

use super::krx_auth_readiness::{
    KRX_API_KEY_ENV_VAR, KRX_ENDPOINT_TEMPLATE_ENV_VAR, KRXAuthReadinessReport,
    KRXAuthReadinessStatus,
};
use super::krx_canonical_validation::KRXCanonicalValidationReport;
use super::krx_official_activation::KRXOfficialEvidenceActivationConfig;
use super::krx_symbol_whitelist::KRXSymbolWhitelist;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum KRXOperatorActionKind {
    SetKRXApiKey,
    SetKRXEndpointTemplate,
    RunProviderReadiness,
    RunProviderReality,
    RunKRXOfficialAcquire,
    RunKRXCollectCandles,
    ProvideKRXCanonicalCsv,
    ProvideKRXProvenance,
    RunKRXPreflight,
    RunOfficialReplication,
    RunCandlePack,
    RunCandleGapMap,
    RunCandleExpand,
    RunJoinAudit,
    RunReadyMatchClose,
    RunCompleteRowCloseV2,
    RunOfficialEvidenceScaleout,
    RunOfficialEvidenceDiversitySweep,
    RunCommitteeOfficialBenchmark,
    RunCorePerformance,
    ReduceScope,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct KRXOperatorAction {
    pub action_id: String,
    pub action_kind: KRXOperatorActionKind,
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

pub fn build_krx_operator_actions(
    config: &KRXOfficialEvidenceActivationConfig,
    auth: &KRXAuthReadinessReport,
    whitelist: &KRXSymbolWhitelist,
    validations: &[KRXCanonicalValidationReport],
    budget_exceeded: bool,
) -> Vec<KRXOperatorAction> {
    let mut actions = Vec::new();
    if matches!(
        auth.readiness_status,
        KRXAuthReadinessStatus::MissingApiKey
            | KRXAuthReadinessStatus::MissingApiKeyAndEndpointTemplate
    ) {
        actions.push(action(
            "set-krx-api-key",
            KRXOperatorActionKind::SetKRXApiKey,
            OperatorActionPriority::Required,
            vec![KRX_API_KEY_ENV_VAR.to_string()],
            "Set KRX_API_KEY locally to enable bounded KRX market-data collection.",
            Some("cargo run --quiet --bin soma_experiment -- krx-auth-readiness --config examples/soma_krx_auth_readiness.toml".to_string()),
            None,
            vec![ReasonCode::KRXOperatorActionPlanBuilt, ReasonCode::MissingApiKey],
        ));
    }
    if matches!(
        auth.readiness_status,
        KRXAuthReadinessStatus::MissingEndpointTemplate
            | KRXAuthReadinessStatus::MissingApiKeyAndEndpointTemplate
    ) {
        actions.push(action(
            "set-krx-endpoint-template",
            KRXOperatorActionKind::SetKRXEndpointTemplate,
            OperatorActionPriority::Required,
            vec![KRX_ENDPOINT_TEMPLATE_ENV_VAR.to_string()],
            "Set KRX_ENDPOINT_TEMPLATE locally with an explicit, secret-safe endpoint template.",
            Some("cargo run --quiet --bin soma_experiment -- krx-auth-readiness --config examples/soma_krx_auth_readiness.toml".to_string()),
            None,
            vec![
                ReasonCode::KRXOperatorActionPlanBuilt,
                ReasonCode::MissingEndpointTemplate,
            ],
        ));
    }
    if whitelist.enabled_entries.len() > config.max_symbols {
        actions.push(action(
            "reduce-krx-scope",
            KRXOperatorActionKind::ReduceScope,
            OperatorActionPriority::Required,
            Vec::new(),
            "Reduce the KRX whitelist to the bounded compact scope before collection or import.",
            Some("cargo run --quiet --bin soma_experiment -- krx-symbol-whitelist --config examples/soma_krx_symbol_whitelist_compact.toml".to_string()),
            None,
            vec![ReasonCode::KRXOperatorActionPlanBuilt, ReasonCode::DeniedByDefault],
        ));
    }
    if validations
        .iter()
        .any(|report| !report.provenance_available)
    {
        actions.push(action(
            "provide-krx-provenance",
            KRXOperatorActionKind::ProvideKRXProvenance,
            OperatorActionPriority::Required,
            Vec::new(),
            "Provide local KRX provenance JSON for every canonical CSV before official readiness claims.",
            None,
            Some("krx_provenance.json".to_string()),
            vec![
                ReasonCode::KRXOperatorActionPlanBuilt,
                ReasonCode::MissingOfficialProvenance,
            ],
        ));
    }
    if validations.iter().any(|report| !report.preflight_available) {
        actions.push(action(
            "run-krx-preflight",
            KRXOperatorActionKind::RunKRXPreflight,
            OperatorActionPriority::Required,
            Vec::new(),
            "Provide or regenerate local KRX preflight reports before using official readiness artifacts.",
            Some("cargo run --quiet --bin soma_experiment -- krx-official-activate --config examples/soma_krx_official_activate_local_import.toml".to_string()),
            Some("preflight_report.json".to_string()),
            vec![
                ReasonCode::KRXOperatorActionPlanBuilt,
                ReasonCode::MissingOfficialPreflight,
            ],
        ));
    }
    if budget_exceeded {
        actions.push(action(
            "reduce-krx-storage-budget",
            KRXOperatorActionKind::ReduceScope,
            OperatorActionPriority::Required,
            Vec::new(),
            "Reduce KRX symbol count or row count to remain within the configured local storage budget.",
            None,
            None,
            vec![ReasonCode::KRXOperatorActionPlanBuilt, ReasonCode::BudgetExceeded],
        ));
    }
    if config.run_official_replication {
        actions.push(action(
            "run-krx-official-replication",
            KRXOperatorActionKind::RunOfficialReplication,
            OperatorActionPriority::Recommended,
            Vec::new(),
            "Run official replication after KRX canonical, provenance, and preflight artifacts are ready.",
            Some("cargo run --quiet --bin soma_experiment -- krx-official-activate --config examples/soma_krx_official_activate_local_import.toml".to_string()),
            Some("official_replication_bundle.json".to_string()),
            vec![ReasonCode::KRXOperatorActionPlanBuilt],
        ));
    }
    if config.run_official_evidence_diversity_sweep {
        actions.push(action(
            "run-krx-diversity-sweep",
            KRXOperatorActionKind::RunOfficialEvidenceDiversitySweep,
            OperatorActionPriority::Recommended,
            Vec::new(),
            "Run the downstream diversity sweep conservatively after official replication artifacts exist.",
            Some("cargo run --quiet --bin soma_experiment -- krx-official-activate --config examples/soma_krx_official_activate_diversity_rerun.toml".to_string()),
            Some("official_evidence_diversity_bundle.json".to_string()),
            vec![ReasonCode::KRXOperatorActionPlanBuilt],
        ));
    }
    if config.run_core_performance {
        actions.push(action(
            "run-krx-core-performance",
            KRXOperatorActionKind::RunCorePerformance,
            OperatorActionPriority::Recommended,
            Vec::new(),
            "Run the research-only core performance scorecard after downstream outcome-linked artifacts are available.",
            Some("cargo run --quiet --bin soma_experiment -- core-performance --config examples/soma_core_performance_diagnostics_only.toml".to_string()),
            Some("core_performance_scorecard.json".to_string()),
            vec![ReasonCode::KRXOperatorActionPlanBuilt],
        ));
    }
    actions.sort_by(|left, right| left.action_id.cmp(&right.action_id));
    actions
}

impl KRXOperatorAction {
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
    action_kind: KRXOperatorActionKind,
    priority: OperatorActionPriority,
    env_var_names: Vec<String>,
    description: &str,
    command_suggestion: Option<String>,
    expected_output_artifact: Option<String>,
    reason_codes: Vec<ReasonCode>,
) -> KRXOperatorAction {
    KRXOperatorAction {
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

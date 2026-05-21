use serde::{Deserialize, Serialize};

use crate::core::ReasonCode;
use crate::data::{ProviderAuthPreflightReport, ProviderKind};
use crate::experiment::OfficialEvidenceExpansionReport;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum OperatorActionPriority {
    Required,
    Recommended,
    Optional,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OperatorAction {
    pub action_id: String,
    pub priority: OperatorActionPriority,
    #[serde(default)]
    pub provider_kind: Option<ProviderKind>,
    pub description: String,
    pub env_var_names: Vec<String>,
    #[serde(default)]
    pub command_suggestion: Option<String>,
    pub safe_to_run: bool,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OperatorActionPlan {
    pub actions: Vec<OperatorAction>,
    pub missing_auth_actions: Vec<String>,
    pub collection_actions: Vec<String>,
    pub evidence_actions: Vec<String>,
    pub next_commands: Vec<String>,
    pub warnings: Vec<String>,
    pub reason_codes: Vec<ReasonCode>,
}

impl OperatorActionPlan {
    pub fn to_text(&self) -> String {
        let mut lines = vec![
            format!(
                "missing_auth_actions={}",
                self.missing_auth_actions.join("|")
            ),
            format!("collection_actions={}", self.collection_actions.join("|")),
            format!("evidence_actions={}", self.evidence_actions.join("|")),
            format!("next_commands={}", self.next_commands.join(" | ")),
            format!("warnings={}", self.warnings.join(" | ")),
        ];
        for action in &self.actions {
            lines.push(format!(
                "action={};priority={:?};provider={};env_var_names={};safe_to_run={};command={}",
                action.action_id,
                action.priority,
                action
                    .provider_kind
                    .map(|provider| format!("{provider:?}"))
                    .unwrap_or_default(),
                action.env_var_names.join("|"),
                action.safe_to_run,
                action.command_suggestion.clone().unwrap_or_default(),
            ));
        }
        lines.join("\n")
    }
}

pub fn build_operator_action_plan(
    auth_preflight_report: &ProviderAuthPreflightReport,
    expansion_report: Option<&OfficialEvidenceExpansionReport>,
    has_generated_collection_plan: bool,
    crypto_ready: bool,
    multi_venue_ready: bool,
) -> OperatorActionPlan {
    let mut actions = Vec::new();

    if auth_preflight_report
        .missing_auth_providers
        .iter()
        .any(|provider| provider == "alphavantage")
    {
        actions.push(OperatorAction {
            action_id: "set-alphavantage-auth".to_string(),
            priority: OperatorActionPriority::Required,
            provider_kind: Some(ProviderKind::AlphaVantage),
            description: "Set ALPHAVANTAGE_API_KEY to enable compact US equity evidence.".to_string(),
            env_var_names: vec!["ALPHAVANTAGE_API_KEY".to_string()],
            command_suggestion: Some(
                "cargo run --bin soma_experiment -- provider-auth-check --config examples/soma_provider_auth_preflight.toml"
                    .to_string(),
            ),
            safe_to_run: true,
            reason_codes: vec![ReasonCode::OperatorActionPlanBuilt, ReasonCode::MissingAuth],
        });
    }

    if auth_preflight_report
        .missing_auth_providers
        .iter()
        .any(|provider| provider == "krx")
    {
        actions.push(OperatorAction {
            action_id: "set-krx-auth".to_string(),
            priority: OperatorActionPriority::Required,
            provider_kind: Some(ProviderKind::KrxOpenApi),
            description: "Set KRX_API_KEY to enable bounded Korean equity evidence.".to_string(),
            env_var_names: vec!["KRX_API_KEY".to_string()],
            command_suggestion: Some(
                "cargo run --bin soma_experiment -- provider-auth-check --config examples/soma_provider_auth_preflight.toml"
                    .to_string(),
            ),
            safe_to_run: true,
            reason_codes: vec![ReasonCode::OperatorActionPlanBuilt, ReasonCode::MissingAuth],
        });
    }

    if auth_preflight_report
        .missing_endpoint_providers
        .iter()
        .any(|provider| provider == "krx")
    {
        actions.push(OperatorAction {
            action_id: "set-krx-endpoint-template".to_string(),
            priority: OperatorActionPriority::Required,
            provider_kind: Some(ProviderKind::KrxOpenApi),
            description: "Set KRX_ENDPOINT_TEMPLATE to enable Korean equity EOD collection.".to_string(),
            env_var_names: vec!["KRX_ENDPOINT_TEMPLATE".to_string()],
            command_suggestion: Some(
                "cargo run --bin soma_experiment -- provider-auth-check --config examples/soma_provider_auth_preflight.toml"
                    .to_string(),
            ),
            safe_to_run: true,
            reason_codes: vec![
                ReasonCode::OperatorActionPlanBuilt,
                ReasonCode::MissingEndpointTemplate,
            ],
        });
    }

    if crypto_ready {
        actions.push(OperatorAction {
            action_id: "run-crypto-only-evidence".to_string(),
            priority: OperatorActionPriority::Recommended,
            provider_kind: Some(ProviderKind::Upbit),
            description: "Run crypto-only official evidence if equity auth is still missing.".to_string(),
            env_var_names: vec![],
            command_suggestion: Some(
                "cargo run --bin soma_experiment -- official-acquire --config examples/soma_official_evidence_acquisition_crypto_only.toml"
                    .to_string(),
            ),
            safe_to_run: true,
            reason_codes: vec![ReasonCode::OperatorActionPlanBuilt],
        });
    }

    if multi_venue_ready {
        actions.push(OperatorAction {
            action_id: "run-multi-venue-evidence".to_string(),
            priority: OperatorActionPriority::Recommended,
            provider_kind: None,
            description: "Run bounded multi-venue evidence after auth preflight is ready.".to_string(),
            env_var_names: vec![],
            command_suggestion: Some(
                "cargo run --bin soma_experiment -- official-acquire --config examples/soma_official_evidence_acquisition_multi_venue.toml"
                    .to_string(),
            ),
            safe_to_run: true,
            reason_codes: vec![ReasonCode::OperatorActionPlanBuilt],
        });
    }

    if has_generated_collection_plan {
        actions.push(OperatorAction {
            action_id: "collect-bounded-official-data".to_string(),
            priority: OperatorActionPriority::Recommended,
            provider_kind: None,
            description: "Run the bounded local-only collection plan generated by Sprint 26.".to_string(),
            env_var_names: vec![],
            command_suggestion: Some(
                "cargo run --bin soma_experiment -- collect-plan --config examples/soma_official_collection_compact.toml"
                    .to_string(),
            ),
            safe_to_run: true,
            reason_codes: vec![ReasonCode::OperatorActionPlanBuilt],
        });
    }

    if expansion_report.is_some() {
        actions.push(OperatorAction {
            action_id: "review-expansion-report".to_string(),
            priority: OperatorActionPriority::Optional,
            provider_kind: None,
            description: "Review the expansion report before broadening scope.".to_string(),
            env_var_names: vec![],
            command_suggestion: Some(
                "cargo run --bin soma_experiment -- evidence-expand --config examples/soma_official_evidence_expansion_multi_venue.toml"
                    .to_string(),
            ),
            safe_to_run: true,
            reason_codes: vec![ReasonCode::OperatorActionPlanBuilt],
        });
    }

    actions.sort_by(|left, right| left.action_id.cmp(&right.action_id));
    let missing_auth_actions = actions
        .iter()
        .filter(|action| !action.env_var_names.is_empty())
        .map(|action| action.action_id.clone())
        .collect::<Vec<_>>();
    let collection_actions = actions
        .iter()
        .filter(|action| {
            action.action_id.contains("collect") || action.action_id.contains("crypto")
        })
        .map(|action| action.action_id.clone())
        .collect::<Vec<_>>();
    let evidence_actions = actions
        .iter()
        .filter(|action| {
            action.action_id.contains("evidence") || action.action_id.contains("review")
        })
        .map(|action| action.action_id.clone())
        .collect::<Vec<_>>();
    let next_commands = actions
        .iter()
        .filter_map(|action| action.command_suggestion.clone())
        .collect::<Vec<_>>();

    OperatorActionPlan {
        actions,
        missing_auth_actions,
        collection_actions,
        evidence_actions,
        next_commands,
        warnings: vec!["All suggested commands stay local-only and research-only.".to_string()],
        reason_codes: vec![ReasonCode::OperatorActionPlanBuilt],
    }
}

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::ReasonCode;

use super::evidence_lane::{EvidenceLaneRunReport, EvidenceLaneStatus};
use super::evidence_lane_runner::EvidenceLaneRunner;
use super::evidence_plan_builder::EvidencePlanBuilder;
use super::executable_evidence_plan::{ExecutableEvidencePlan, ExecutableEvidencePlanConfig};
use super::lane_storage::{
    ProviderRealityStorageReport, build_lane_storage_budget_report,
    build_provider_reality_storage_report,
};
use super::operator_action::{OperatorAction, OperatorActionPlan, OperatorActionPriority};
use super::provider_reality::ProviderRealityReport;
use super::readiness_matrix::{EvidenceReadinessMatrix, build_evidence_readiness_matrix};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProviderRealityEvidenceFinalStatus {
    NoRunnableLanes,
    CryptoOnlyRan,
    EodEvidenceRan,
    MultiVenueEvidenceRan,
    ResearchOnlyYFinanceRan,
    MissingAuth,
    MissingApproval,
    MissingEntitlement,
    CoreBlocked,
    BudgetBlocked,
    NeedMoreEvidence,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProviderRealityEvidenceRecommendation {
    RunCryptoOnlyEvidence,
    WaitForKrxApproval,
    SetKrxAuth,
    SetDataGoKrAuth,
    SetAlphaVantageAuth,
    SetAlpacaAuth,
    MoreOfficialEvidence,
    ImproveDataFirst,
    ImproveSignalModelFirst,
    ImproveRiskGovernorFirst,
    BuildSequenceDatasetFirst,
    HoldCurrentScope,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProviderRealityEvidenceReport {
    pub plan_id: String,
    pub executable_plan: ExecutableEvidencePlan,
    pub lane_reports: Vec<EvidenceLaneRunReport>,
    pub readiness_matrix: EvidenceReadinessMatrix,
    pub storage_report: ProviderRealityStorageReport,
    pub operator_action_plan: OperatorActionPlan,
    pub final_status: ProviderRealityEvidenceFinalStatus,
    pub final_recommendation: ProviderRealityEvidenceRecommendation,
    pub blockers: Vec<String>,
    pub warnings: Vec<String>,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ProviderRealityEvidenceExecutor;

impl ProviderRealityEvidenceExecutor {
    pub fn run(
        &self,
        config: &ExecutableEvidencePlanConfig,
    ) -> Result<ProviderRealityEvidenceReport, String> {
        config.validate()?;
        let plan = if let Some(path) = &config.provider_reality_report_path {
            let report = ProviderRealityReport::from_json_path(Path::new(path))?;
            EvidencePlanBuilder::default().from_provider_reality(&report, config)?
        } else {
            EvidencePlanBuilder::default().from_explicit_lanes(config)?
        };
        let runner = EvidenceLaneRunner::default();
        let mut lane_reports = plan
            .lanes
            .iter()
            .map(|lane| runner.run_lane(lane, config))
            .collect::<Vec<_>>();
        lane_reports.sort_by(|left, right| left.lane_id.cmp(&right.lane_id));
        let storage_report = build_provider_reality_storage_report(
            plan.lanes
                .iter()
                .map(|lane| {
                    let actual = lane_reports
                        .iter()
                        .find(|report| report.lane_id == lane.lane_id)
                        .map(|report| report.storage_bytes);
                    build_lane_storage_budget_report(lane, actual)
                })
                .collect(),
            config.max_total_bytes,
        );
        let readiness_matrix = build_evidence_readiness_matrix(&plan, &lane_reports);
        let operator_action_plan = build_operator_action_plan(&plan, &lane_reports);
        let final_status = classify_final_status(&plan, &lane_reports, &storage_report);
        let final_recommendation =
            classify_final_recommendation(&operator_action_plan, final_status);
        let blockers = build_blockers(&plan, &lane_reports, &storage_report);
        let warnings = build_warnings(&plan, &storage_report);
        Ok(ProviderRealityEvidenceReport {
            plan_id: config.plan_id.clone(),
            executable_plan: plan,
            lane_reports,
            readiness_matrix,
            storage_report,
            operator_action_plan,
            final_status,
            final_recommendation,
            blockers,
            warnings,
            reason_codes: vec![ReasonCode::ProviderRealityEvidenceExecutorBuilt],
        })
    }
}

impl ProviderRealityEvidenceReport {
    pub fn to_json_string(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|err| err.to_string())
    }

    pub fn to_text(&self) -> String {
        let mut lines = vec![
            format!("plan_id={}", self.plan_id),
            format!("final_status={:?}", self.final_status),
            format!("final_recommendation={:?}", self.final_recommendation),
            format!("blockers={}", self.blockers.join("|")),
            format!("warnings={}", self.warnings.join("|")),
        ];
        lines.push(self.executable_plan.to_text());
        lines.push(self.readiness_matrix.to_text());
        lines.push(self.operator_action_plan.to_text());
        lines.join("\n")
    }

    pub fn write_to_dir(&self, output_dir: &Path) -> Result<PathBuf, String> {
        fs::create_dir_all(output_dir).map_err(|err| err.to_string())?;
        let json_path = output_dir.join("provider_reality_evidence_report.json");
        fs::write(&json_path, self.to_json_string()?).map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("provider_reality_evidence_report.txt"),
            self.to_text(),
        )
        .map_err(|err| err.to_string())?;
        Ok(json_path)
    }
}

fn classify_final_status(
    plan: &ExecutableEvidencePlan,
    lane_reports: &[EvidenceLaneRunReport],
    storage_report: &ProviderRealityStorageReport,
) -> ProviderRealityEvidenceFinalStatus {
    if storage_report.budget_exceeded && plan.runnable_lanes.is_empty() {
        return ProviderRealityEvidenceFinalStatus::BudgetBlocked;
    }
    let successful = lane_reports
        .iter()
        .filter(|report| report.lane_status == EvidenceLaneStatus::RanSuccessfully)
        .collect::<Vec<_>>();
    if successful.is_empty() {
        if plan
            .skipped_lanes
            .iter()
            .any(|lane| lane.lane_status == EvidenceLaneStatus::SkippedMissingApproval)
        {
            ProviderRealityEvidenceFinalStatus::MissingApproval
        } else if plan
            .skipped_lanes
            .iter()
            .any(|lane| lane.lane_status == EvidenceLaneStatus::SkippedMissingAuth)
        {
            ProviderRealityEvidenceFinalStatus::MissingAuth
        } else if plan.skipped_lanes.iter().any(|lane| {
            matches!(
                lane.lane_status,
                EvidenceLaneStatus::SkippedMissingEndpointTemplate
                    | EvidenceLaneStatus::SkippedMissingEntitlement
                    | EvidenceLaneStatus::SkippedIncompatibleFreshness
            )
        }) {
            ProviderRealityEvidenceFinalStatus::MissingEntitlement
        } else if lane_reports
            .iter()
            .any(|report| report.lane_status == EvidenceLaneStatus::SkippedCoreBlocked)
        {
            ProviderRealityEvidenceFinalStatus::CoreBlocked
        } else {
            ProviderRealityEvidenceFinalStatus::NoRunnableLanes
        }
    } else if successful.iter().all(|report| {
        matches!(
            report.source_kind,
            crate::data::EvidenceSourceKind::YFinanceResearch
        )
    }) {
        ProviderRealityEvidenceFinalStatus::ResearchOnlyYFinanceRan
    } else if successful
        .iter()
        .all(|report| matches!(report.provider_kind, Some(crate::data::ProviderKind::Upbit)))
    {
        ProviderRealityEvidenceFinalStatus::CryptoOnlyRan
    } else if successful.len() > 1 {
        ProviderRealityEvidenceFinalStatus::MultiVenueEvidenceRan
    } else {
        ProviderRealityEvidenceFinalStatus::EodEvidenceRan
    }
}

fn classify_final_recommendation(
    operator_action_plan: &OperatorActionPlan,
    final_status: ProviderRealityEvidenceFinalStatus,
) -> ProviderRealityEvidenceRecommendation {
    if operator_action_plan
        .actions
        .iter()
        .any(|action| action.action_id == "wait-krx-approval")
    {
        return ProviderRealityEvidenceRecommendation::WaitForKrxApproval;
    }
    if operator_action_plan
        .actions
        .iter()
        .any(|action| action.action_id == "set-krx-auth")
    {
        return ProviderRealityEvidenceRecommendation::SetKrxAuth;
    }
    if operator_action_plan
        .actions
        .iter()
        .any(|action| action.action_id == "set-datagokr-auth")
    {
        return ProviderRealityEvidenceRecommendation::SetDataGoKrAuth;
    }
    if operator_action_plan
        .actions
        .iter()
        .any(|action| action.action_id == "set-alphavantage-auth")
    {
        return ProviderRealityEvidenceRecommendation::SetAlphaVantageAuth;
    }
    if operator_action_plan
        .actions
        .iter()
        .any(|action| action.action_id == "set-alpaca-auth")
    {
        return ProviderRealityEvidenceRecommendation::SetAlpacaAuth;
    }
    match final_status {
        ProviderRealityEvidenceFinalStatus::CryptoOnlyRan => {
            ProviderRealityEvidenceRecommendation::RunCryptoOnlyEvidence
        }
        ProviderRealityEvidenceFinalStatus::BudgetBlocked => {
            ProviderRealityEvidenceRecommendation::ImproveDataFirst
        }
        ProviderRealityEvidenceFinalStatus::ResearchOnlyYFinanceRan => {
            ProviderRealityEvidenceRecommendation::BuildSequenceDatasetFirst
        }
        ProviderRealityEvidenceFinalStatus::EodEvidenceRan
        | ProviderRealityEvidenceFinalStatus::MultiVenueEvidenceRan => {
            ProviderRealityEvidenceRecommendation::HoldCurrentScope
        }
        _ => ProviderRealityEvidenceRecommendation::MoreOfficialEvidence,
    }
}

fn build_blockers(
    plan: &ExecutableEvidencePlan,
    lane_reports: &[EvidenceLaneRunReport],
    storage_report: &ProviderRealityStorageReport,
) -> Vec<String> {
    let mut blockers = plan
        .skipped_lanes
        .iter()
        .map(|lane| format!("{}:{:?}", lane.lane_id, lane.lane_status))
        .collect::<Vec<_>>();
    if storage_report.budget_exceeded {
        blockers.push("storage budget exceeded".to_string());
    }
    if lane_reports
        .iter()
        .any(|report| report.lane_status == EvidenceLaneStatus::SkippedCoreBlocked)
    {
        blockers.push("core-check blocked benchmark execution".to_string());
    }
    blockers.sort();
    blockers.dedup();
    blockers
}

fn build_warnings(
    plan: &ExecutableEvidencePlan,
    storage_report: &ProviderRealityStorageReport,
) -> Vec<String> {
    let mut warnings = plan
        .lanes
        .iter()
        .flat_map(|lane| lane.warnings.iter().cloned())
        .collect::<Vec<_>>();
    if !storage_report.budget_exceeded
        && storage_report.total_estimated_bytes.saturating_mul(100)
            >= plan
                .storage_budget_summary
                .total_estimated_bytes
                .saturating_mul(85)
    {
        warnings.push("storage budget near limit".to_string());
    }
    warnings.sort();
    warnings.dedup();
    warnings
}

fn build_operator_action_plan(
    plan: &ExecutableEvidencePlan,
    lane_reports: &[EvidenceLaneRunReport],
) -> OperatorActionPlan {
    let mut actions = Vec::new();
    for lane in &plan.skipped_lanes {
        match (lane.provider_kind, lane.lane_status) {
            (
                Some(crate::data::ProviderKind::KrxOpenApi),
                EvidenceLaneStatus::SkippedMissingApproval,
            ) => {
                actions.push(action(
                    "wait-krx-approval",
                    OperatorActionPriority::Required,
                    lane.provider_kind,
                    "Wait for KRX approval before claiming Korean official evidence readiness.",
                    &[],
                    Some("cargo run --bin soma_experiment -- provider-readiness --config examples/soma_provider_readiness.toml"),
                    vec![ReasonCode::MissingApproval],
                ));
            }
            (
                Some(crate::data::ProviderKind::KrxOpenApi),
                EvidenceLaneStatus::SkippedMissingAuth,
            ) => {
                actions.push(action(
                    "set-krx-auth",
                    OperatorActionPriority::Required,
                    lane.provider_kind,
                    "Set KRX auth env vars for bounded Korean evidence.",
                    &["KRX_API_KEY"],
                    Some("cargo run --bin soma_experiment -- provider-auth-check --config examples/soma_provider_auth_preflight.toml"),
                    vec![ReasonCode::MissingAuth],
                ));
            }
            (
                Some(crate::data::ProviderKind::KrxOpenApi),
                EvidenceLaneStatus::SkippedMissingEndpointTemplate,
            ) => {
                actions.push(action(
                    "set-krx-endpoint-template",
                    OperatorActionPriority::Required,
                    lane.provider_kind,
                    "Set KRX endpoint template for bounded Korean EOD collection.",
                    &["KRX_ENDPOINT_TEMPLATE"],
                    Some("cargo run --bin soma_experiment -- provider-auth-check --config examples/soma_provider_auth_preflight.toml"),
                    vec![ReasonCode::MissingEndpointTemplate],
                ));
            }
            (
                Some(crate::data::ProviderKind::DataGoKrFscStockPrice),
                EvidenceLaneStatus::SkippedMissingAuth,
            ) => {
                actions.push(action(
                    "set-datagokr-auth",
                    OperatorActionPriority::Required,
                    lane.provider_kind,
                    "Set data.go.kr service key for Korean fallback evidence.",
                    &["DATA_GO_KR_SERVICE_KEY"],
                    Some("cargo run --bin soma_experiment -- provider-readiness --config examples/soma_provider_readiness.toml"),
                    vec![ReasonCode::MissingAuth],
                ));
            }
            (
                Some(crate::data::ProviderKind::AlphaVantage),
                EvidenceLaneStatus::SkippedMissingAuth,
            ) => {
                actions.push(action(
                    "set-alphavantage-auth",
                    OperatorActionPriority::Required,
                    lane.provider_kind,
                    "Set AlphaVantage API key for bounded US EOD evidence.",
                    &["ALPHAVANTAGE_API_KEY"],
                    Some("cargo run --bin soma_experiment -- provider-readiness --config examples/soma_provider_readiness.toml"),
                    vec![ReasonCode::MissingAuth],
                ));
            }
            (Some(crate::data::ProviderKind::Alpaca), EvidenceLaneStatus::SkippedMissingAuth) => {
                actions.push(action(
                    "set-alpaca-auth",
                    OperatorActionPriority::Required,
                    lane.provider_kind,
                    "Set Alpaca market-data keys for bounded US realtime research.",
                    &["ALPACA_API_KEY_ID", "ALPACA_API_SECRET_KEY"],
                    Some("cargo run --bin soma_experiment -- provider-readiness --config examples/soma_provider_readiness.toml"),
                    vec![ReasonCode::MissingAuth],
                ));
            }
            (_, EvidenceLaneStatus::SkippedMissingEntitlement) => {
                actions.push(action(
                    "configure-realtime-entitlement",
                    OperatorActionPriority::Recommended,
                    lane.provider_kind,
                    "Buy or configure bounded realtime entitlement before claiming fuller US realtime coverage.",
                    &[],
                    Some("cargo run --bin soma_experiment -- provider-reality --config examples/soma_provider_reality.toml"),
                    vec![ReasonCode::MissingPremiumEntitlement],
                ));
            }
            _ => {}
        }
    }

    if plan
        .runnable_lanes
        .iter()
        .any(|lane| matches!(lane.provider_kind, Some(crate::data::ProviderKind::Upbit)))
    {
        actions.push(action(
            "use-upbit-crypto-only",
            OperatorActionPriority::Recommended,
            Some(crate::data::ProviderKind::Upbit),
            "Run crypto-only evidence while equity auth gaps remain.",
            &[],
            Some("cargo run --bin soma_experiment -- evidence-execute --config examples/soma_evidence_plan_crypto_only.toml"),
            vec![ReasonCode::OfficialApiCollected],
        ));
    }
    if plan.lanes.iter().any(|lane| {
        matches!(
            lane.provider_subject,
            crate::data::ProviderDataSubject::YFinanceResearch
        )
    }) {
        actions.push(action(
            "use-yfinance-research-only",
            OperatorActionPriority::Optional,
            None,
            "Keep yfinance only in research comparison or prototype lanes.",
            &[],
            Some("cargo run --bin soma_experiment -- yahoo-research --config examples/soma_provider_reality.toml"),
            vec![ReasonCode::YFinanceResearchOnly],
        ));
    }
    if lane_reports
        .iter()
        .any(|report| report.benchmark_report.is_some())
    {
        actions.push(action(
            "run-core-check",
            OperatorActionPriority::Recommended,
            None,
            "Run core-check before broadening any benchmark scope.",
            &[],
            Some("cargo run --bin soma_experiment -- core-check"),
            vec![ReasonCode::CoreReadinessBuilt],
        ));
    }

    actions.sort_by(|left, right| left.action_id.cmp(&right.action_id));
    actions.dedup_by(|left, right| left.action_id == right.action_id);
    OperatorActionPlan {
        missing_auth_actions: actions
            .iter()
            .filter(|action| !action.env_var_names.is_empty())
            .map(|action| action.action_id.clone())
            .collect(),
        collection_actions: actions
            .iter()
            .filter(|action| {
                action.action_id.contains("upbit") || action.action_id.contains("core")
            })
            .map(|action| action.action_id.clone())
            .collect(),
        evidence_actions: actions
            .iter()
            .map(|action| action.action_id.clone())
            .collect(),
        next_commands: actions
            .iter()
            .filter_map(|action| action.command_suggestion.clone())
            .collect(),
        warnings: vec!["All suggested commands stay local-only and research-only.".to_string()],
        actions,
        reason_codes: vec![ReasonCode::OperatorActionPlanBuilt],
    }
}

fn action(
    action_id: &str,
    priority: OperatorActionPriority,
    provider_kind: Option<crate::data::ProviderKind>,
    description: &str,
    env_var_names: &[&str],
    command_suggestion: Option<&str>,
    mut reason_codes: Vec<ReasonCode>,
) -> OperatorAction {
    reason_codes.push(ReasonCode::OperatorActionPlanBuilt);
    OperatorAction {
        action_id: action_id.to_string(),
        priority,
        provider_kind,
        description: description.to_string(),
        env_var_names: env_var_names
            .iter()
            .map(|value| (*value).to_string())
            .collect(),
        command_suggestion: command_suggestion.map(|value| value.to_string()),
        safe_to_run: true,
        reason_codes,
    }
}

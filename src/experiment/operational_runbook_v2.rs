use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_reason_codes};

use super::kis_market_data_smoke::{
    KISMarketDataEvidenceSmokeFinalStatus, KISMarketDataEvidenceSmokeReport,
};
use crate::ui::ControlTowerAutoRefreshReport;

fn default_output_root() -> String {
    "target/sprint58/operational_runbook_v2".to_string()
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OperationalRunbookV2Config {
    pub runbook_id: String,
    #[serde(default)]
    pub smoke_report_paths: Vec<String>,
    #[serde(default)]
    pub control_tower_auto_refresh_report_paths: Vec<String>,
    #[serde(default = "default_output_root")]
    pub output_root: String,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum OperationalRunbookV2StepKind {
    KISAuthClosure,
    KISMarketDataDryRun,
    KISCollectionPlanV2,
    KISMarketDataSmoke,
    KISEvidenceDepthRun,
    TrinityOperationalLoop,
    ControlTowerRefresh,
    DashboardOpen,
    OwnerReviewQueue,
    CorePerformance,
    StopDueToBlocker,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum OperationalRunbookV2FinalStatus {
    #[default]
    ReadyToRun,
    BlockedByKISAuth,
    BlockedByEndpointPolicy,
    BlockedByMissingEvidence,
    BlockedBySecretSafety,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OperationalRunbookV2Step {
    pub step_id: String,
    pub step_kind: OperationalRunbookV2StepKind,
    #[serde(default)]
    pub command_suggestion: Option<String>,
    #[serde(default)]
    pub expected_artifact: Option<String>,
    pub blocked: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct OperationalRunbookV2Report {
    pub runbook_id: String,
    #[serde(default)]
    pub steps: Vec<OperationalRunbookV2Step>,
    #[serde(default)]
    pub ordered_steps: Vec<String>,
    pub required_steps: usize,
    pub optional_steps: usize,
    pub blocked_steps: usize,
    pub primary_next_step: String,
    #[serde(default)]
    pub command_suggestions: Vec<String>,
    #[serde(default)]
    pub expected_artifacts: Vec<String>,
    pub final_status: OperationalRunbookV2FinalStatus,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct OperationalRunbookV2Runner;

impl Default for OperationalRunbookV2Config {
    fn default() -> Self {
        Self {
            runbook_id: "sprint58-operational-runbook-v2".to_string(),
            smoke_report_paths: Vec::new(),
            control_tower_auto_refresh_report_paths: Vec::new(),
            output_root: default_output_root(),
            reason_codes: vec![ReasonCode::DeterministicPath],
        }
    }
}

impl OperationalRunbookV2Config {
    pub fn from_toml_path(path: &Path) -> Result<Self, String> {
        let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
        toml::from_str(&text).map_err(|err| err.to_string())
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.runbook_id.trim().is_empty() {
            return Err("operational runbook v2 id must not be empty".to_string());
        }
        if self
            .smoke_report_paths
            .iter()
            .chain(self.control_tower_auto_refresh_report_paths.iter())
            .chain(std::iter::once(&self.output_root))
            .any(|path| path.contains("://"))
        {
            return Err("operational runbook v2 paths must be local".to_string());
        }
        Ok(())
    }

    pub fn artifact_dir(&self) -> PathBuf {
        PathBuf::from(&self.output_root).join(&self.runbook_id)
    }
}

impl OperationalRunbookV2Report {
    pub fn to_text(&self) -> String {
        let mut lines = vec![
            "paper_only_warning=operational runbook v2 emits local research-only and paper-only commands".to_string(),
            format!("runbook_id={}", self.runbook_id),
            format!("ordered_steps={}", self.ordered_steps.join("|")),
            format!("required_steps={}", self.required_steps),
            format!("optional_steps={}", self.optional_steps),
            format!("blocked_steps={}", self.blocked_steps),
            format!("primary_next_step={}", self.primary_next_step),
            format!("final_status={:?}", self.final_status),
        ];
        for step in &self.steps {
            lines.push(format!(
                "step={};kind={:?};blocked={};command={};expected={}",
                step.step_id,
                step.step_kind,
                step.blocked,
                step.command_suggestion.clone().unwrap_or_default(),
                step.expected_artifact.clone().unwrap_or_default()
            ));
        }
        lines.push(format!(
            "reason_codes={}",
            self.reason_codes
                .iter()
                .map(|reason| format!("{reason:?}"))
                .collect::<Vec<_>>()
                .join("|")
        ));
        lines.join("\n")
    }

    pub fn write_to_dir(&self, output_dir: &Path) -> Result<PathBuf, String> {
        fs::create_dir_all(output_dir).map_err(|err| err.to_string())?;
        let text_path = output_dir.join("operational_runbook_v2.txt");
        fs::write(&text_path, self.to_text()).map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("operational_runbook_v2.json"),
            serde_json::to_string_pretty(self).map_err(|err| err.to_string())?,
        )
        .map_err(|err| err.to_string())?;
        Ok(text_path)
    }
}

impl OperationalRunbookV2Runner {
    pub fn run(
        &self,
        config: &OperationalRunbookV2Config,
    ) -> Result<OperationalRunbookV2Report, String> {
        config.validate()?;
        let smoke_report =
            load_latest::<KISMarketDataEvidenceSmokeReport>(&config.smoke_report_paths)?;
        let refresh_report = load_latest::<ControlTowerAutoRefreshReport>(
            &config.control_tower_auto_refresh_report_paths,
        )?;
        Ok(self.run_with_reports(config, smoke_report.as_ref(), refresh_report.as_ref())?)
    }

    pub fn run_with_reports(
        &self,
        config: &OperationalRunbookV2Config,
        smoke_report: Option<&KISMarketDataEvidenceSmokeReport>,
        refresh_report: Option<&ControlTowerAutoRefreshReport>,
    ) -> Result<OperationalRunbookV2Report, String> {
        config.validate()?;
        let mut steps = vec![
            step(
                "step-01-kis-auth-closure",
                OperationalRunbookV2StepKind::KISAuthClosure,
                "cargo run --quiet --bin soma_experiment -- kis-auth-close --config examples/soma_kis_auth_close.toml",
                "target/sprint58/kis_auth_closure/",
                false,
            ),
            step(
                "step-02-kis-market-data-dry-run",
                OperationalRunbookV2StepKind::KISMarketDataDryRun,
                "cargo run --quiet --bin soma_experiment -- kis-market-data-dry-run --config examples/soma_kis_market_data_dry_run.toml",
                "target/sprint58/kis_market_data_dry_run/",
                false,
            ),
            step(
                "step-03-kis-collection-plan-v2",
                OperationalRunbookV2StepKind::KISCollectionPlanV2,
                "cargo run --quiet --bin soma_experiment -- kis-collection-plan-v2 --config examples/soma_kis_collection_plan_v2_fixture.toml",
                "target/sprint58/kis_collection_plan_v2/",
                false,
            ),
            step(
                "step-04-kis-market-data-smoke",
                OperationalRunbookV2StepKind::KISMarketDataSmoke,
                "cargo run --quiet --bin soma_experiment -- kis-market-data-smoke --config examples/soma_kis_market_data_smoke_fixture.toml",
                "target/sprint58/kis_market_data_smoke/",
                false,
            ),
            step(
                "step-05-kis-evidence-depth-run",
                OperationalRunbookV2StepKind::KISEvidenceDepthRun,
                "cargo run --quiet --bin soma_experiment -- kis-evidence-depth-run --config examples/soma_kis_evidence_depth_run.toml",
                "target/sprint57/kis_evidence_depth/",
                false,
            ),
            step(
                "step-06-trinity-operational-loop",
                OperationalRunbookV2StepKind::TrinityOperationalLoop,
                "cargo run --quiet --bin soma_experiment -- trinity-operational-loop --config examples/soma_trinity_operational_loop_kis.toml",
                "target/sprint56/sprint56-operational-loop-kis/",
                false,
            ),
            step(
                "step-07-control-tower-refresh",
                OperationalRunbookV2StepKind::ControlTowerRefresh,
                "cargo run --quiet --bin soma_experiment -- control-tower-auto-refresh --config examples/soma_control_tower_auto_refresh.toml",
                "target/sprint58/control_tower_auto_refresh/",
                false,
            ),
            step(
                "step-08-dashboard-open",
                OperationalRunbookV2StepKind::DashboardOpen,
                "cargo run --quiet --bin soma_experiment -- dashboard-open --config examples/soma_control_tower_v1_kis.toml",
                "target/sprint58/control_tower_auto_refresh/",
                false,
            ),
            step(
                "step-09-owner-review-queue",
                OperationalRunbookV2StepKind::OwnerReviewQueue,
                "cargo run --quiet --bin soma_experiment -- owner-review-queue --config examples/soma_owner_review_queue.toml",
                "owner review queue text output",
                false,
            ),
            step(
                "step-10-core-performance",
                OperationalRunbookV2StepKind::CorePerformance,
                "cargo run --quiet --bin soma_experiment -- core-performance --config examples/soma_core_performance_diagnostics_only.toml",
                "target/core_performance_diagnostics_only/",
                false,
            ),
        ];

        let final_status = match smoke_report.map(|report| report.final_status) {
            Some(
                KISMarketDataEvidenceSmokeFinalStatus::KISAuthMissing
                | KISMarketDataEvidenceSmokeFinalStatus::KISBaseUrlMissing,
            ) => {
                steps.push(step(
                    "step-99-stop-due-to-blocker",
                    OperationalRunbookV2StepKind::StopDueToBlocker,
                    "",
                    "",
                    true,
                ));
                OperationalRunbookV2FinalStatus::BlockedByKISAuth
            }
            Some(KISMarketDataEvidenceSmokeFinalStatus::EndpointPolicyBlocked) => {
                steps.push(step(
                    "step-99-stop-due-to-blocker",
                    OperationalRunbookV2StepKind::StopDueToBlocker,
                    "",
                    "",
                    true,
                ));
                OperationalRunbookV2FinalStatus::BlockedByEndpointPolicy
            }
            Some(
                KISMarketDataEvidenceSmokeFinalStatus::StillNeedKISMarketData
                | KISMarketDataEvidenceSmokeFinalStatus::StillNeedOutcomeLinkDepth
                | KISMarketDataEvidenceSmokeFinalStatus::StillNeedCounterfactualDepth
                | KISMarketDataEvidenceSmokeFinalStatus::NoImprovement,
            ) => OperationalRunbookV2FinalStatus::BlockedByMissingEvidence,
            Some(KISMarketDataEvidenceSmokeFinalStatus::DiagnosticOnly) | None => {
                OperationalRunbookV2FinalStatus::DiagnosticOnly
            }
            _ => {
                if refresh_report.is_some_and(|report| {
                    !report.secret_redaction_report.secret_leaks_detected
                        && !report.secret_redaction_report.token_like_values_detected
                        && !report.secret_redaction_report.account_like_fields_detected
                        && !report.secret_redaction_report.order_like_fields_detected
                }) {
                    OperationalRunbookV2FinalStatus::ReadyToRun
                } else if refresh_report.is_some() {
                    OperationalRunbookV2FinalStatus::BlockedBySecretSafety
                } else {
                    OperationalRunbookV2FinalStatus::ReadyToRun
                }
            }
        };

        let ordered_steps = steps
            .iter()
            .map(|step| step.step_id.clone())
            .collect::<Vec<_>>();
        let command_suggestions = steps
            .iter()
            .filter_map(|step| step.command_suggestion.clone())
            .filter(|command| !command.is_empty())
            .collect::<Vec<_>>();
        let expected_artifacts = steps
            .iter()
            .filter_map(|step| step.expected_artifact.clone())
            .filter(|artifact| !artifact.is_empty())
            .collect::<Vec<_>>();
        let blocked_steps = steps.iter().filter(|step| step.blocked).count();
        let report = OperationalRunbookV2Report {
            runbook_id: config.runbook_id.clone(),
            primary_next_step: steps
                .iter()
                .find(|step| !step.blocked)
                .map(|step| step.step_id.clone())
                .unwrap_or_else(|| "step-99-stop-due-to-blocker".to_string()),
            required_steps: steps.len().saturating_sub(2),
            optional_steps: 2,
            blocked_steps,
            steps,
            ordered_steps,
            command_suggestions,
            expected_artifacts,
            final_status,
            reason_codes: stable_reason_codes(
                &[
                    config.reason_codes.clone(),
                    vec![ReasonCode::OperationalRunbookV2Built],
                ]
                .concat(),
            ),
        };
        report.write_to_dir(&config.artifact_dir())?;
        Ok(report)
    }
}

fn step(
    step_id: &str,
    step_kind: OperationalRunbookV2StepKind,
    command_suggestion: &str,
    expected_artifact: &str,
    blocked: bool,
) -> OperationalRunbookV2Step {
    OperationalRunbookV2Step {
        step_id: step_id.to_string(),
        step_kind,
        command_suggestion: if command_suggestion.is_empty() {
            None
        } else {
            Some(command_suggestion.to_string())
        },
        expected_artifact: if expected_artifact.is_empty() {
            None
        } else {
            Some(expected_artifact.to_string())
        },
        blocked,
        reason_codes: stable_reason_codes(&[ReasonCode::DeterministicPath]),
    }
}

fn load_latest<T: for<'de> Deserialize<'de>>(paths: &[String]) -> Result<Option<T>, String> {
    let mut latest = None;
    for path in paths {
        let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
        latest = Some(serde_json::from_str(&text).map_err(|err| err.to_string())?);
    }
    Ok(latest)
}

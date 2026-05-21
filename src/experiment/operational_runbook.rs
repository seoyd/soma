use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_hash_string, stable_reason_codes};
use crate::league::TrinityCommitteeOperationalLoopConfig;
use crate::ui::ControlTowerRefreshConfig;

use super::kis_auth_readiness::{
    KIS_APP_KEY_ENV_VAR, KIS_APP_SECRET_ENV_VAR, KIS_BASE_URL_ENV_VAR,
};
use super::kis_evidence_depth::KISEvidenceDepthRunConfig;

fn default_output_root() -> String {
    "target/sprint57/runbook".to_string()
}

fn default_true() -> bool {
    true
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OperationalRunbookConfig {
    pub runbook_id: String,
    #[serde(default)]
    pub kis_evidence_depth_config_path: Option<String>,
    #[serde(default)]
    pub control_tower_refresh_config_path: Option<String>,
    #[serde(default)]
    pub trinity_loop_config_path: Option<String>,
    #[serde(default = "default_output_root")]
    pub output_root: String,
    #[serde(default = "default_true")]
    pub include_commands: bool,
    #[serde(default = "default_true")]
    pub include_expected_artifacts: bool,
    #[serde(default = "default_true")]
    pub include_blockers: bool,
    #[serde(default = "default_true")]
    pub include_risk_notes: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum OperationalRunbookStepKind {
    KISAuthCheck,
    KISMarketDataActivate,
    KISCandleSufficiency,
    KISOutcomeLinkClose,
    KISEvidenceDepthRun,
    TrinityOperationalLoop,
    ControlTowerRefresh,
    DashboardOpen,
    OwnerReviewQueue,
    CorePerformance,
    #[default]
    StopDueToBlocker,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct OperationalRunbookStep {
    pub step_id: String,
    pub step_kind: OperationalRunbookStepKind,
    #[serde(default)]
    pub command_suggestion: Option<String>,
    #[serde(default)]
    pub expected_artifact: Option<String>,
    #[serde(default)]
    pub preconditions: Vec<String>,
    #[serde(default)]
    pub blockers: Vec<String>,
    pub safe_to_run: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum OperationalRunbookFinalStatus {
    #[default]
    ReadyToRun,
    BlockedByKISAuth,
    BlockedByMissingEvidence,
    BlockedByRisk,
    DiagnosticOnly,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct OperationalRunbookReport {
    pub runbook_id: String,
    #[serde(default)]
    pub steps: Vec<OperationalRunbookStep>,
    pub required_steps: usize,
    pub optional_steps: usize,
    pub blocked_steps: usize,
    pub final_status: OperationalRunbookFinalStatus,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
    pub fingerprint: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct OperationalRunbookRunner;

impl Default for OperationalRunbookConfig {
    fn default() -> Self {
        Self {
            runbook_id: "sprint57-operational-runbook".to_string(),
            kis_evidence_depth_config_path: None,
            control_tower_refresh_config_path: None,
            trinity_loop_config_path: None,
            output_root: default_output_root(),
            include_commands: true,
            include_expected_artifacts: true,
            include_blockers: true,
            include_risk_notes: true,
            reason_codes: vec![ReasonCode::DeterministicPath],
        }
    }
}

impl OperationalRunbookConfig {
    pub fn from_toml_path(path: &Path) -> Result<Self, String> {
        let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
        toml::from_str(&text).map_err(|err| err.to_string())
    }

    pub fn to_toml_string(&self) -> Result<String, String> {
        toml::to_string_pretty(self).map_err(|err| err.to_string())
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.runbook_id.trim().is_empty() {
            return Err("operational runbook id must not be empty".to_string());
        }
        if self
            .all_paths()
            .iter()
            .chain(std::iter::once(&self.output_root))
            .any(|path| path.contains("://"))
        {
            return Err("operational runbook paths must be local".to_string());
        }
        Ok(())
    }

    pub fn artifact_dir(&self) -> PathBuf {
        PathBuf::from(&self.output_root).join(&self.runbook_id)
    }

    pub fn all_paths(&self) -> Vec<String> {
        [
            self.kis_evidence_depth_config_path.clone(),
            self.control_tower_refresh_config_path.clone(),
            self.trinity_loop_config_path.clone(),
        ]
        .into_iter()
        .flatten()
        .collect()
    }
}

impl OperationalRunbookReport {
    pub fn diagnostic_only(runbook_id: impl Into<String>) -> Self {
        let mut report = Self {
            runbook_id: runbook_id.into(),
            steps: Vec::new(),
            required_steps: 0,
            optional_steps: 0,
            blocked_steps: 0,
            final_status: OperationalRunbookFinalStatus::DiagnosticOnly,
            reason_codes: stable_reason_codes(&[
                ReasonCode::ResearchOnlyOverride,
                ReasonCode::DeterministicPath,
            ]),
            fingerprint: String::new(),
        };
        report.stabilize();
        report
    }

    pub fn stabilize(&mut self) {
        self.steps
            .sort_by(|left, right| left.step_id.cmp(&right.step_id));
        for step in &mut self.steps {
            step.preconditions.sort();
            step.preconditions.dedup();
            step.blockers.sort();
            step.blockers.dedup();
            step.reason_codes = stable_reason_codes(&step.reason_codes);
        }
        self.required_steps = self
            .steps
            .iter()
            .filter(|step| is_required_step(step))
            .count();
        self.optional_steps = self.steps.len().saturating_sub(self.required_steps);
        self.blocked_steps = self.steps.iter().filter(|step| !step.safe_to_run).count();
        self.reason_codes = stable_reason_codes(&self.reason_codes);
        self.fingerprint = stable_hash_string(&serde_json::to_string(self).unwrap_or_default());
    }

    pub fn to_json_string(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|err| err.to_string())
    }

    pub fn from_json_path(path: &Path) -> Result<Self, String> {
        let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
        serde_json::from_str(&text).map_err(|err| err.to_string())
    }

    pub fn to_text(&self) -> String {
        let mut lines = vec![
            "paper_only_warning=operational runbook emits local research and paper-only commands only"
                .to_string(),
            format!("runbook_id={}", self.runbook_id),
            format!("required_steps={}", self.required_steps),
            format!("optional_steps={}", self.optional_steps),
            format!("blocked_steps={}", self.blocked_steps),
            format!("final_status={:?}", self.final_status),
        ];
        lines.extend(self.steps.iter().map(|step| {
            format!(
                "step={};kind={:?};safe_to_run={};command={};expected={};blockers={}",
                step.step_id,
                step.step_kind,
                step.safe_to_run,
                step.command_suggestion.clone().unwrap_or_default(),
                step.expected_artifact.clone().unwrap_or_default(),
                step.blockers.join(" | ")
            )
        }));
        lines.push(format!("fingerprint={}", self.fingerprint));
        lines.join("\n")
    }

    pub fn write_to_dir(&self, output_dir: &Path) -> Result<PathBuf, String> {
        fs::create_dir_all(output_dir).map_err(|err| err.to_string())?;
        fs::write(output_dir.join("operational_runbook.txt"), self.to_text())
            .map_err(|err| err.to_string())?;
        let json_path = output_dir.join("operational_runbook.json");
        fs::write(&json_path, self.to_json_string()?).map_err(|err| err.to_string())?;
        Ok(json_path)
    }
}

impl OperationalRunbookRunner {
    pub fn run(
        &self,
        config: &OperationalRunbookConfig,
    ) -> Result<OperationalRunbookReport, String> {
        config.validate()?;

        let auth_ready = env::var(KIS_APP_KEY_ENV_VAR).is_ok()
            && env::var(KIS_APP_SECRET_ENV_VAR).is_ok()
            && env::var(KIS_BASE_URL_ENV_VAR).is_ok();

        let depth_config = config
            .kis_evidence_depth_config_path
            .as_deref()
            .map(Path::new)
            .map(KISEvidenceDepthRunConfig::from_toml_path)
            .transpose()?;
        let refresh_config = config
            .control_tower_refresh_config_path
            .as_deref()
            .map(Path::new)
            .map(ControlTowerRefreshConfig::from_toml_path)
            .transpose()?;
        let trinity_config = config
            .trinity_loop_config_path
            .as_deref()
            .map(Path::new)
            .map(TrinityCommitteeOperationalLoopConfig::from_toml_path)
            .transpose()?;

        let mut steps = Vec::new();
        steps.push(step(
            "step-01-kis-auth-check",
            OperationalRunbookStepKind::KISAuthCheck,
            Some(
                "cargo run --quiet --bin soma_experiment -- kis-auth-readiness --config examples/soma_kis_auth_readiness.toml"
                    .to_string(),
            ),
            Some("KIS auth readiness text output".to_string()),
            vec!["KIS_APP_KEY, KIS_APP_SECRET, KIS_BASE_URL must be present in env".to_string()],
            if auth_ready {
                Vec::new()
            } else {
                vec!["KIS env auth is missing".to_string()]
            },
            true,
            vec![ReasonCode::KISAuthReadinessBuilt, ReasonCode::DeterministicPath],
        ));

        let market_data_blockers = if auth_ready {
            Vec::new()
        } else {
            vec!["KIS auth must pass first".to_string()]
        };
        steps.push(step(
            "step-02-kis-market-data-activate",
            OperationalRunbookStepKind::KISMarketDataActivate,
            Some(
                "cargo run --quiet --bin soma_experiment -- kis-market-data-activate --config examples/soma_kis_market_data_activate_local_import.toml"
                    .to_string(),
            ),
            Some("target/sprint51/kis_market_data_activate_local_import/".to_string()),
            vec!["auth check completed".to_string()],
            market_data_blockers,
            auth_ready,
            vec![ReasonCode::KISMarketDataActivationRan, ReasonCode::DeterministicPath],
        ));

        let depth_config_path = config.kis_evidence_depth_config_path.clone();
        let depth_blockers = if depth_config.is_some() {
            Vec::new()
        } else {
            vec!["kis evidence depth config is missing".to_string()]
        };
        steps.push(step(
            "step-03-kis-candle-sufficiency",
            OperationalRunbookStepKind::KISCandleSufficiency,
            Some(
                "cargo run --quiet --bin soma_experiment -- kis-candle-sufficiency --config examples/soma_kis_candle_sufficiency.toml"
                    .to_string(),
            ),
            Some("target/sprint51/kis_candle_sufficiency/".to_string()),
            vec!["market data activation artifact exists".to_string()],
            blockers_union(&depth_blockers, &if auth_ready { Vec::new() } else { vec!["KIS auth must pass first".to_string()] }),
            auth_ready && depth_config.is_some(),
            vec![ReasonCode::KISCandleSufficiencyBuilt, ReasonCode::DeterministicPath],
        ));
        steps.push(step(
            "step-04-kis-outcome-link-close",
            OperationalRunbookStepKind::KISOutcomeLinkClose,
            Some(
                "cargo run --quiet --bin soma_experiment -- kis-outcome-link-close --config examples/soma_kis_outcome_link_close.toml"
                    .to_string(),
            ),
            Some("target/sprint51/kis_outcome_link_close/".to_string()),
            vec!["candle sufficiency output exists".to_string()],
            blockers_union(&depth_blockers, &if auth_ready { Vec::new() } else { vec!["KIS auth must pass first".to_string()] }),
            auth_ready && depth_config.is_some(),
            vec![ReasonCode::KISOutcomeLinkClosureBuilt, ReasonCode::DeterministicPath],
        ));

        let depth_expected = depth_config.as_ref().map(|item| {
            item.artifact_dir()
                .join("kis_evidence_depth_report.txt")
                .display()
                .to_string()
        });
        steps.push(step(
            "step-05-kis-evidence-depth-run",
            OperationalRunbookStepKind::KISEvidenceDepthRun,
            depth_config_path.as_ref().map(|path| {
                format!(
                    "cargo run --quiet --bin soma_experiment -- kis-evidence-depth-run --config {path}"
                )
            }),
            depth_expected,
            vec!["local evidence artifacts prepared".to_string()],
            depth_blockers.clone(),
            depth_config.is_some(),
            vec![ReasonCode::OfficialEvidenceCounted, ReasonCode::DeterministicPath],
        ));

        let trinity_blockers = if trinity_config.is_some() {
            Vec::new()
        } else {
            vec!["trinity operational loop config is missing".to_string()]
        };
        let trinity_expected = trinity_config.as_ref().map(|item| {
            item.artifact_dir()
                .join("trinity_operational_loop_report.json")
                .display()
                .to_string()
        });
        steps.push(step(
            "step-06-trinity-operational-loop",
            OperationalRunbookStepKind::TrinityOperationalLoop,
            config.trinity_loop_config_path.as_ref().map(|path| {
                format!(
                    "cargo run --quiet --bin soma_experiment -- trinity-operational-loop --config {path}"
                )
            }),
            trinity_expected,
            vec!["evidence depth report available".to_string()],
            trinity_blockers.clone(),
            trinity_config.is_some(),
            vec![ReasonCode::PaperExecutionOnly, ReasonCode::DeterministicPath],
        ));

        let refresh_blockers = if refresh_config.is_some() {
            Vec::new()
        } else {
            vec!["control tower refresh config is missing".to_string()]
        };
        let refresh_expected = refresh_config.as_ref().map(|item| {
            item.artifact_dir()
                .join("dashboard_state_v1.json")
                .display()
                .to_string()
        });
        steps.push(step(
            "step-07-control-tower-refresh",
            OperationalRunbookStepKind::ControlTowerRefresh,
            config.control_tower_refresh_config_path.as_ref().map(|path| {
                format!(
                    "cargo run --quiet --bin soma_experiment -- control-tower-refresh --config {path}"
                )
            }),
            refresh_expected,
            vec!["trinity loop summary available".to_string()],
            refresh_blockers.clone(),
            refresh_config.is_some(),
            vec![ReasonCode::DashboardRendered, ReasonCode::DeterministicPath],
        ));

        let dashboard_open_expected = refresh_config.as_ref().map(|item| {
            item.artifact_dir()
                .join("dashboard_v1.html")
                .display()
                .to_string()
        });
        steps.push(step(
            "step-08-dashboard-open",
            OperationalRunbookStepKind::DashboardOpen,
            refresh_config.as_ref().map(|item| {
                let config_path = item
                    .control_tower_v1_config_path
                    .clone()
                    .unwrap_or_else(|| "examples/soma_control_tower_v1_kis.toml".to_string());
                format!(
                    "cargo run --quiet --bin soma_experiment -- dashboard-open --config {config_path}"
                )
            }),
            dashboard_open_expected,
            vec!["refreshed dashboard html exists".to_string()],
            refresh_blockers.clone(),
            refresh_config.is_some(),
            vec![ReasonCode::DashboardRendered, ReasonCode::DeterministicPath],
        ));

        steps.push(step(
            "step-09-owner-review-queue",
            OperationalRunbookStepKind::OwnerReviewQueue,
            Some(
                "cargo run --quiet --bin soma_experiment -- owner-review-queue --config examples/soma_owner_review_queue.toml"
                    .to_string(),
            ),
            Some("owner review queue text output".to_string()),
            vec!["run after dashboard refresh if owner items exist".to_string()],
            Vec::new(),
            true,
            vec![ReasonCode::OwnerReviewQueueBuilt, ReasonCode::DeterministicPath],
        ));

        steps.push(step(
            "step-10-core-performance",
            OperationalRunbookStepKind::CorePerformance,
            Some(
                "cargo run --quiet --bin soma_experiment -- core-performance --config examples/soma_core_performance_diagnostics_only.toml"
                    .to_string(),
            ),
            Some("target/core_performance_diagnostics_only/".to_string()),
            vec!["use for conservative scorecard rerun only".to_string()],
            Vec::new(),
            true,
            vec![ReasonCode::CorePerformanceScorecardBuilt, ReasonCode::DeterministicPath],
        ));

        let mut reason_codes = vec![ReasonCode::DeterministicPath, ReasonCode::LocalFileOnly];
        let final_status = if !auth_ready {
            reason_codes.push(ReasonCode::MissingAuth);
            OperationalRunbookFinalStatus::BlockedByKISAuth
        } else if depth_config.is_none() {
            reason_codes.push(ReasonCode::MissingFile);
            OperationalRunbookFinalStatus::BlockedByMissingEvidence
        } else {
            OperationalRunbookFinalStatus::ReadyToRun
        };

        if !matches!(final_status, OperationalRunbookFinalStatus::ReadyToRun) {
            steps.push(step(
                "step-99-stop-due-to-blocker",
                OperationalRunbookStepKind::StopDueToBlocker,
                None,
                None,
                vec!["resolve blockers before continuing".to_string()],
                match final_status {
                    OperationalRunbookFinalStatus::BlockedByKISAuth => {
                        vec!["KIS env auth is missing".to_string()]
                    }
                    OperationalRunbookFinalStatus::BlockedByMissingEvidence => {
                        vec!["kis evidence depth config is missing".to_string()]
                    }
                    OperationalRunbookFinalStatus::BlockedByRisk => {
                        vec!["risk governor requires manual review".to_string()]
                    }
                    OperationalRunbookFinalStatus::DiagnosticOnly
                    | OperationalRunbookFinalStatus::ReadyToRun => Vec::new(),
                },
                false,
                vec![
                    ReasonCode::ResearchOnlyOverride,
                    ReasonCode::DeterministicPath,
                ],
            ));
        }

        let mut report = OperationalRunbookReport {
            runbook_id: config.runbook_id.clone(),
            steps,
            required_steps: 0,
            optional_steps: 0,
            blocked_steps: 0,
            final_status,
            reason_codes: stable_reason_codes(&reason_codes),
            fingerprint: String::new(),
        };
        report.stabilize();
        report.write_to_dir(&config.artifact_dir())?;
        Ok(report)
    }
}

fn is_required_step(step: &OperationalRunbookStep) -> bool {
    matches!(
        step.step_kind,
        OperationalRunbookStepKind::KISAuthCheck
            | OperationalRunbookStepKind::KISMarketDataActivate
            | OperationalRunbookStepKind::KISCandleSufficiency
            | OperationalRunbookStepKind::KISOutcomeLinkClose
            | OperationalRunbookStepKind::KISEvidenceDepthRun
            | OperationalRunbookStepKind::TrinityOperationalLoop
            | OperationalRunbookStepKind::ControlTowerRefresh
            | OperationalRunbookStepKind::StopDueToBlocker
    )
}

fn step(
    step_id: &str,
    step_kind: OperationalRunbookStepKind,
    command_suggestion: Option<String>,
    expected_artifact: Option<String>,
    preconditions: Vec<String>,
    blockers: Vec<String>,
    safe_to_run: bool,
    reason_codes: Vec<ReasonCode>,
) -> OperationalRunbookStep {
    OperationalRunbookStep {
        step_id: step_id.to_string(),
        step_kind,
        command_suggestion,
        expected_artifact,
        preconditions,
        blockers,
        safe_to_run,
        reason_codes: stable_reason_codes(&reason_codes),
    }
}

fn blockers_union(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    merged.extend(right.iter().cloned());
    merged.sort();
    merged.dedup();
    merged
}

use std::fs;
use std::path::{Path, PathBuf};

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_hash_string, stable_ordered_strings, stable_reason_codes};
use crate::experiment::{
    KISEvidenceDepthFinalRecommendation, KISEvidenceDepthReport, OperationalRunbookReport,
    OperationalRunbookStepKind,
};
use crate::league::{PaperPositionLifecycleReport, TrinityOperationalLoopReport};

use super::{
    control_tower_health::summarize_control_tower_health,
    control_tower_v1::{ControlTowerRefreshPlanner, ControlTowerV1Builder, ControlTowerV1Config},
    dashboard_events::{
        AuditTimelinePanel, DashboardEvent, DashboardEventKind, DashboardEventSeverity,
    },
    dashboard_panels::PaperPositionPanel,
    dashboard_v1_renderer::{
        DashboardV1RenderReport, DashboardV1RenderStatus, DashboardV1Renderer,
    },
    next_action_panel::{NextActionItem, NextActionKind, NextActionPanel, NextActionPriority},
};

fn default_output_root() -> String {
    "target/sprint57/control_tower_refresh".to_string()
}

fn default_true() -> bool {
    true
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ControlTowerRefreshConfig {
    pub refresh_id: String,
    #[serde(default)]
    pub control_tower_v1_config_path: Option<String>,
    #[serde(default)]
    pub dashboard_source_config_path: Option<String>,
    #[serde(default)]
    pub kis_evidence_depth_report_paths: Vec<String>,
    #[serde(default)]
    pub trinity_loop_report_paths: Vec<String>,
    #[serde(default)]
    pub paper_lifecycle_report_paths: Vec<String>,
    #[serde(default)]
    pub owner_review_queue_paths: Vec<String>,
    #[serde(default)]
    pub core_scorecard_paths: Vec<String>,
    #[serde(default)]
    pub operational_runbook_report_paths: Vec<String>,
    #[serde(default = "default_output_root")]
    pub output_root: String,
    #[serde(default = "default_true")]
    pub render_html: bool,
    #[serde(default = "default_true")]
    pub render_json: bool,
    #[serde(default = "default_true")]
    pub render_text: bool,
    #[serde(default = "default_true")]
    pub generate_next_actions: bool,
    #[serde(default = "default_true")]
    pub generate_owner_action_drafts: bool,
    #[serde(default = "default_max_events")]
    pub max_events: usize,
    #[serde(default = "default_max_candidates")]
    pub max_candidates: usize,
    #[serde(default = "default_max_owner_inputs")]
    pub max_owner_inputs: usize,
    #[serde(default = "default_max_paper_positions")]
    pub max_paper_positions: usize,
    #[serde(default = "default_max_bytes")]
    pub max_bytes: usize,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

fn default_max_events() -> usize {
    1000
}

fn default_max_candidates() -> usize {
    200
}

fn default_max_owner_inputs() -> usize {
    200
}

fn default_max_paper_positions() -> usize {
    100
}

fn default_max_bytes() -> usize {
    20_000_000
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ControlTowerRefreshStatus {
    Refreshed,
    RefreshedWithWarnings,
    MissingSourceReports,
    SecretRedactionFailed,
    UnsafeControlDetected,
    NoImprovement,
    #[default]
    DiagnosticOnly,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ControlTowerRefreshReport {
    pub refresh_id: String,
    pub dashboard_state_path: String,
    #[serde(default)]
    pub dashboard_html_path: Option<String>,
    #[serde(default)]
    pub dashboard_text_path: Option<String>,
    #[serde(default)]
    pub next_actions_path: Option<String>,
    #[serde(default)]
    pub owner_drafts_dir: Option<String>,
    pub panel_count: usize,
    pub candidate_count: usize,
    pub paper_position_count: usize,
    pub owner_pending_count: usize,
    pub event_count: usize,
    pub primary_bottleneck: String,
    pub primary_next_action: String,
    pub secret_redaction_passed: bool,
    pub unsafe_control_detected: bool,
    pub refresh_status: ControlTowerRefreshStatus,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
    pub fingerprint: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ControlTowerRefreshOutput {
    pub state: super::control_tower_v1::ControlTowerV1State,
    pub render_report: DashboardV1RenderReport,
    pub report: ControlTowerRefreshReport,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ControlTowerRefreshRunner;

impl Default for ControlTowerRefreshConfig {
    fn default() -> Self {
        Self {
            refresh_id: "sprint57-control-tower-refresh".to_string(),
            control_tower_v1_config_path: None,
            dashboard_source_config_path: None,
            kis_evidence_depth_report_paths: Vec::new(),
            trinity_loop_report_paths: Vec::new(),
            paper_lifecycle_report_paths: Vec::new(),
            owner_review_queue_paths: Vec::new(),
            core_scorecard_paths: Vec::new(),
            operational_runbook_report_paths: Vec::new(),
            output_root: default_output_root(),
            render_html: true,
            render_json: true,
            render_text: true,
            generate_next_actions: true,
            generate_owner_action_drafts: true,
            max_events: default_max_events(),
            max_candidates: default_max_candidates(),
            max_owner_inputs: default_max_owner_inputs(),
            max_paper_positions: default_max_paper_positions(),
            max_bytes: default_max_bytes(),
            reason_codes: vec![ReasonCode::DeterministicPath],
        }
    }
}

impl ControlTowerRefreshConfig {
    pub fn from_toml_path(path: &Path) -> Result<Self, String> {
        let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
        toml::from_str(&text).map_err(|err| err.to_string())
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.refresh_id.trim().is_empty() {
            return Err("control tower refresh id must not be empty".to_string());
        }
        if self
            .all_paths()
            .iter()
            .chain(std::iter::once(&self.output_root))
            .any(|path| path.contains("://"))
        {
            return Err("control tower refresh paths must be local".to_string());
        }
        if self.max_events == 0 || self.max_events > 1000 {
            return Err("control tower refresh max_events must be between 1 and 1000".to_string());
        }
        if self.max_candidates == 0 || self.max_candidates > 200 {
            return Err(
                "control tower refresh max_candidates must be between 1 and 200".to_string(),
            );
        }
        if self.max_owner_inputs == 0 || self.max_owner_inputs > 200 {
            return Err(
                "control tower refresh max_owner_inputs must be between 1 and 200".to_string(),
            );
        }
        if self.max_paper_positions == 0 || self.max_paper_positions > 100 {
            return Err(
                "control tower refresh max_paper_positions must be between 1 and 100".to_string(),
            );
        }
        if self.max_bytes == 0 || self.max_bytes > 20_000_000 {
            return Err(
                "control tower refresh max_bytes must be between 1 and 20000000".to_string(),
            );
        }
        Ok(())
    }

    pub fn artifact_dir(&self) -> PathBuf {
        PathBuf::from(&self.output_root).join(&self.refresh_id)
    }

    pub fn all_paths(&self) -> Vec<String> {
        let mut paths = Vec::new();
        paths.extend(self.control_tower_v1_config_path.clone());
        paths.extend(self.dashboard_source_config_path.clone());
        paths.extend(self.kis_evidence_depth_report_paths.clone());
        paths.extend(self.trinity_loop_report_paths.clone());
        paths.extend(self.paper_lifecycle_report_paths.clone());
        paths.extend(self.owner_review_queue_paths.clone());
        paths.extend(self.core_scorecard_paths.clone());
        paths.extend(self.operational_runbook_report_paths.clone());
        stable_ordered_strings(&paths)
    }
}

impl ControlTowerRefreshReport {
    pub fn diagnostic_only(refresh_id: impl Into<String>, output_root: impl Into<String>) -> Self {
        let refresh_id = refresh_id.into();
        let base = PathBuf::from(output_root.into()).join(&refresh_id);
        let mut report = Self {
            refresh_id,
            dashboard_state_path: base.join("dashboard_state_v1.json").display().to_string(),
            dashboard_html_path: Some(base.join("dashboard_v1.html").display().to_string()),
            dashboard_text_path: Some(base.join("dashboard_v1.txt").display().to_string()),
            next_actions_path: Some(
                base.join("dashboard_next_actions.txt")
                    .display()
                    .to_string(),
            ),
            owner_drafts_dir: Some(base.join("owner_action_drafts").display().to_string()),
            panel_count: 0,
            candidate_count: 0,
            paper_position_count: 0,
            owner_pending_count: 0,
            event_count: 0,
            primary_bottleneck: "DiagnosticOnly".to_string(),
            primary_next_action: "NoAction".to_string(),
            secret_redaction_passed: true,
            unsafe_control_detected: false,
            refresh_status: ControlTowerRefreshStatus::DiagnosticOnly,
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
        [
            "read_only_warning=control tower refresh remains local-only, deterministic, paper-only, and read-only"
                .to_string(),
            format!("refresh_id={}", self.refresh_id),
            format!("dashboard_state_path={}", self.dashboard_state_path),
            format!(
                "dashboard_html_path={}",
                self.dashboard_html_path.clone().unwrap_or_default()
            ),
            format!(
                "dashboard_text_path={}",
                self.dashboard_text_path.clone().unwrap_or_default()
            ),
            format!(
                "next_actions_path={}",
                self.next_actions_path.clone().unwrap_or_default()
            ),
            format!(
                "owner_drafts_dir={}",
                self.owner_drafts_dir.clone().unwrap_or_default()
            ),
            format!("panel_count={}", self.panel_count),
            format!("candidate_count={}", self.candidate_count),
            format!("paper_position_count={}", self.paper_position_count),
            format!("owner_pending_count={}", self.owner_pending_count),
            format!("event_count={}", self.event_count),
            format!("primary_bottleneck={}", self.primary_bottleneck),
            format!("primary_next_action={}", self.primary_next_action),
            format!("secret_redaction_passed={}", self.secret_redaction_passed),
            format!("unsafe_control_detected={}", self.unsafe_control_detected),
            format!("refresh_status={:?}", self.refresh_status),
            format!("fingerprint={}", self.fingerprint),
        ]
        .join("\n")
    }

    pub fn write_to_dir(&self, output_dir: &Path) -> Result<PathBuf, String> {
        fs::create_dir_all(output_dir).map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("control_tower_refresh_report.txt"),
            self.to_text(),
        )
        .map_err(|err| err.to_string())?;
        let json_path = output_dir.join("control_tower_refresh_report.json");
        fs::write(&json_path, self.to_json_string()?).map_err(|err| err.to_string())?;
        Ok(json_path)
    }
}

impl ControlTowerRefreshRunner {
    pub fn run(
        &self,
        config: &ControlTowerRefreshConfig,
        config_path: Option<&Path>,
        depth_report: Option<&KISEvidenceDepthReport>,
        runbook_report: Option<&OperationalRunbookReport>,
    ) -> Result<ControlTowerRefreshOutput, String> {
        config.validate()?;

        let mut missing_sources = Vec::new();
        let derived_config = derive_control_tower_config(config, &mut missing_sources)?;
        let mut state = ControlTowerV1Builder::default().build(&derived_config, None)?;

        let loaded_depth = depth_report.cloned().or_else(|| {
            load_latest::<KISEvidenceDepthReport>(
                &config.kis_evidence_depth_report_paths,
                &mut missing_sources,
            )
        });
        let loaded_loop = load_latest::<TrinityOperationalLoopReport>(
            &config.trinity_loop_report_paths,
            &mut missing_sources,
        );
        let loaded_runbook = runbook_report.cloned().or_else(|| {
            load_latest::<OperationalRunbookReport>(
                &config.operational_runbook_report_paths,
                &mut missing_sources,
            )
        });
        let loaded_paper = load_latest::<PaperPositionLifecycleReport>(
            &config.paper_lifecycle_report_paths,
            &mut missing_sources,
        );

        if let Some(report) = loaded_depth.as_ref() {
            apply_depth_report(&mut state, report, runbook_report);
        }
        if let Some(report) = loaded_loop.as_ref() {
            apply_loop_report(&mut state, report);
        }
        if let Some(report) = loaded_paper.as_ref() {
            state.paper_lifecycle_panel = report.to_panel().into();
            state.paper_position_panel = report.to_panel();
        }
        if config.generate_next_actions {
            if let Some(runbook) = loaded_runbook.as_ref() {
                state.next_action_panel = build_runbook_next_action_panel(runbook);
            } else if let Some(report) = loaded_depth.as_ref() {
                state.next_action_panel = build_depth_next_action_panel(report, config_path);
            }
        }

        state.generated_from_reports.extend(config.all_paths());
        state.generated_from_reports = stable_ordered_strings(&state.generated_from_reports);
        state.refresh_planner = build_refresh_planner_for_refresh(config, config_path);
        state.health_summary = summarize_control_tower_health(
            &state.kis_monitor_panel,
            &state.evidence_panel,
            &state.owner_panel,
            &state.risk_panel,
            &state.candidate_panel,
            &state.next_action_panel,
            &state.refresh_planner,
            &state.warnings,
            &state.blockers,
            true,
        );
        state = state.with_fingerprint();

        let render_report = DashboardV1Renderer::default().render(&state, &derived_config)?;
        let refresh_status = derive_refresh_status(
            &render_report,
            loaded_depth.as_ref(),
            !missing_sources.is_empty(),
        );
        let dashboard_state_path = render_report.json_path.clone().unwrap_or_else(|| {
            derived_config
                .artifact_dir()
                .join("dashboard_state_v1.json")
                .display()
                .to_string()
        });
        let dashboard_text_path = render_report.text_path.clone().or_else(|| {
            Some(
                derived_config
                    .artifact_dir()
                    .join("dashboard_v1.txt")
                    .display()
                    .to_string(),
            )
        });
        let next_actions_path = Some(
            derived_config
                .artifact_dir()
                .join("dashboard_next_actions.txt")
                .display()
                .to_string(),
        );
        let primary_next_action = state
            .next_action_panel
            .primary_next_action
            .command_suggestion
            .clone()
            .unwrap_or_else(|| {
                format!(
                    "{:?}",
                    state.next_action_panel.primary_next_action.action_kind
                )
            });

        let mut reason_codes = vec![ReasonCode::DashboardRendered, ReasonCode::DeterministicPath];
        if !render_report.secret_redaction_passed {
            reason_codes.push(ReasonCode::DashboardSecretRejected);
        }
        if render_report.unsafe_control_detected {
            reason_codes.push(ReasonCode::LiveSafetyUnsafePath);
        }
        if !missing_sources.is_empty() {
            reason_codes.push(ReasonCode::DashboardReportMissing);
        }

        let mut report = ControlTowerRefreshReport {
            refresh_id: config.refresh_id.clone(),
            dashboard_state_path,
            dashboard_html_path: render_report.html_path.clone(),
            dashboard_text_path,
            next_actions_path,
            owner_drafts_dir: render_report.owner_action_draft_dir.clone(),
            panel_count: render_report.panel_count,
            candidate_count: state.candidate_panel.candidates.len(),
            paper_position_count: state.paper_position_panel.open_positions.len()
                + state.paper_position_panel.closed_positions.len()
                + state.paper_position_panel.risk_closed_positions.len(),
            owner_pending_count: state.owner_panel.pending_review_items.len(),
            event_count: state.audit_timeline.events.len(),
            primary_bottleneck: state.health_summary.current_primary_bottleneck.clone(),
            primary_next_action,
            secret_redaction_passed: render_report.secret_redaction_passed,
            unsafe_control_detected: render_report.unsafe_control_detected,
            refresh_status,
            reason_codes: stable_reason_codes(&reason_codes),
            fingerprint: String::new(),
        };
        report.stabilize();
        report.write_to_dir(&config.artifact_dir())?;

        Ok(ControlTowerRefreshOutput {
            state,
            render_report,
            report,
        })
    }
}

fn derive_control_tower_config(
    config: &ControlTowerRefreshConfig,
    missing_sources: &mut Vec<String>,
) -> Result<ControlTowerV1Config, String> {
    let mut base = if let Some(path) = &config.control_tower_v1_config_path {
        ControlTowerV1Config::from_toml_path(Path::new(path))?
    } else {
        if config.dashboard_source_config_path.is_none() {
            missing_sources
                .push("missing base control tower or dashboard source config".to_string());
        }
        ControlTowerV1Config::default()
    };
    base.control_tower_id = config.refresh_id.clone();
    base.output_root = config.output_root.clone();
    base.render_html = config.render_html;
    base.render_json = config.render_json;
    base.render_text = config.render_text;
    base.generate_owner_action_drafts = config.generate_owner_action_drafts;
    base.max_events = config.max_events;
    base.max_candidates = config.max_candidates;
    base.max_owner_inputs = config.max_owner_inputs;
    base.max_paper_positions = config.max_paper_positions;
    if let Some(path) = &config.dashboard_source_config_path {
        base.dashboard_source_config_paths.push(path.clone());
    }
    base.owner_review_queue_paths
        .extend(config.owner_review_queue_paths.clone());
    base.core_scorecard_paths
        .extend(config.core_scorecard_paths.clone());
    base.operational_loop_report_paths
        .extend(config.trinity_loop_report_paths.clone());
    base.reason_codes = stable_reason_codes(
        &base
            .reason_codes
            .iter()
            .chain(config.reason_codes.iter())
            .cloned()
            .collect::<Vec<_>>(),
    );
    Ok(base)
}

fn build_refresh_planner_for_refresh(
    config: &ControlTowerRefreshConfig,
    config_path: Option<&Path>,
) -> ControlTowerRefreshPlanner {
    let artifact_dir = config.artifact_dir();
    let expected_output_artifacts = vec![
        artifact_dir
            .join("dashboard_state_v1.json")
            .display()
            .to_string(),
        artifact_dir.join("dashboard_v1.html").display().to_string(),
        artifact_dir.join("dashboard_v1.txt").display().to_string(),
        artifact_dir
            .join("dashboard_next_actions.txt")
            .display()
            .to_string(),
        artifact_dir
            .join("owner_action_drafts")
            .display()
            .to_string(),
    ];
    let missing_outputs = expected_output_artifacts
        .iter()
        .filter(|path| !Path::new(path).exists())
        .count();
    let refresh_command = format!(
        "cargo run --quiet --bin soma_experiment -- control-tower-refresh --config {}",
        config_path
            .map(|path| path.display().to_string())
            .unwrap_or_else(
                || "examples/soma_control_tower_refresh_after_kis_depth.toml".to_string()
            )
    );
    let mut planner = ControlTowerRefreshPlanner {
        watcher_mode: super::control_tower_v1::ControlTowerWatcherMode::RefreshPlannerOnly,
        tracked_inputs: config.all_paths().len(),
        missing_outputs,
        refresh_commands: vec![refresh_command],
        expected_output_artifacts,
        reason_codes: vec![ReasonCode::DeterministicPath],
    };
    planner.stabilize();
    planner
}

fn load_latest<T: DeserializeOwned>(
    paths: &[String],
    missing_sources: &mut Vec<String>,
) -> Option<T> {
    let mut latest = None;
    for path in paths {
        match fs::read_to_string(path) {
            Ok(text) => match serde_json::from_str::<T>(&text) {
                Ok(parsed) => latest = Some(parsed),
                Err(_) => missing_sources.push(format!("failed to parse {}", path)),
            },
            Err(_) => missing_sources.push(format!("missing {}", path)),
        }
    }
    latest
}

fn apply_depth_report(
    state: &mut super::control_tower_v1::ControlTowerV1State,
    report: &KISEvidenceDepthReport,
    runbook_report: Option<&OperationalRunbookReport>,
) {
    state.evidence_panel.official_rows_before = report.official_rows_before;
    state.evidence_panel.official_rows_after = Some(report.official_rows_after);
    state.evidence_panel.official_rows = report.official_rows_after;
    state.evidence_panel.complete_rows_before = report.complete_rows_before;
    state.evidence_panel.complete_rows_after = Some(report.complete_rows_after);
    state.evidence_panel.official_complete_rows = report.complete_rows_after;
    state.evidence_panel.outcome_links_before = report.outcome_links_before;
    state.evidence_panel.outcome_links_after = Some(report.outcome_links_after);
    state.evidence_panel.outcome_links = report.outcome_links_after;
    state.evidence_panel.counterfactuals_before = Some(
        report.no_trade_counterfactuals_before.unwrap_or_default()
            + report
                .risk_denied_counterfactuals_before
                .unwrap_or_default(),
    );
    state.evidence_panel.counterfactuals_after =
        Some(report.no_trade_counterfactuals_after + report.risk_denied_counterfactuals_after);
    state.evidence_panel.no_trade_counterfactuals = report.no_trade_counterfactuals_after;
    state.evidence_panel.risk_denied_counterfactuals = report.risk_denied_counterfactuals_after;
    state.evidence_panel.sufficiency_status = format!("{:?}", report.depth_status);
    state.evidence_panel.diversity_status = report
        .diversity_status_after
        .clone()
        .unwrap_or_else(|| "Unknown".to_string());
    state.evidence_panel.current_bottleneck = report.primary_bottleneck_after.clone();
    state.evidence_panel.next_recommended_action = format!("{:?}", report.final_recommendation);

    state.kis_monitor_panel.latest_depth_run_id = Some(report.run_id.clone());
    state.kis_monitor_panel.latest_depth_status = Some(format!("{:?}", report.depth_status));
    state.kis_monitor_panel.latest_outcome_closure = Some(format!(
        "{} -> {}",
        report.outcome_links_before.unwrap_or_default(),
        report.outcome_links_after
    ));
    state.kis_monitor_panel.official_row_count = report.official_rows_after;
    state.kis_monitor_panel.complete_rows = report.complete_rows_after;
    state.kis_monitor_panel.outcome_links = report.outcome_links_after;
    state.kis_monitor_panel.counterfactuals =
        report.no_trade_counterfactuals_after + report.risk_denied_counterfactuals_after;
    state.kis_monitor_panel.latest_next_command = runbook_report
        .and_then(|runbook| {
            runbook
                .steps
                .iter()
                .find(|step| step.safe_to_run)
                .and_then(|step| step.command_suggestion.clone())
        })
        .or_else(|| recommendation_to_command(report.final_recommendation));
}

fn apply_loop_report(
    state: &mut super::control_tower_v1::ControlTowerV1State,
    report: &TrinityOperationalLoopReport,
) {
    state.operational_loop_panel = report.operational_loop_panel.clone();
    state.operational_loop_panel.last_loop_run_id = Some(report.loop_id.clone());
    state.trinity_status_panel = report.trinity_status_panel.clone();
    state.candidate_lifecycle_panel = report.candidate_lifecycle_panel.clone();
    state.paper_lifecycle_panel = report.paper_lifecycle_panel.clone();
    state.paper_position_panel = report.paper_position_lifecycle_report.to_panel();
    state.audit_timeline =
        map_operational_timeline(&report.operational_audit_timeline, &report.loop_id);
}

fn build_runbook_next_action_panel(runbook: &OperationalRunbookReport) -> NextActionPanel {
    let mut recommended_actions = Vec::new();
    let mut blocked_actions = Vec::new();
    for step in &runbook.steps {
        let item = NextActionItem {
            action_id: step.step_id.clone(),
            action_kind: map_step_kind(step.step_kind),
            priority: if step.safe_to_run {
                NextActionPriority::Required
            } else {
                NextActionPriority::Recommended
            },
            command_suggestion: step.command_suggestion.clone(),
            expected_output_artifact: step.expected_artifact.clone(),
            safe_to_run: step.safe_to_run,
            reason_codes: step.reason_codes.clone(),
        };
        if step.safe_to_run {
            recommended_actions.push(item);
        } else {
            blocked_actions.push(item);
        }
    }
    let primary_next_action = recommended_actions
        .first()
        .cloned()
        .or_else(|| blocked_actions.first().cloned())
        .unwrap_or_default();
    let mut panel = NextActionPanel {
        primary_next_action,
        recommended_actions,
        blocked_actions,
        operator_actions: Vec::new(),
        reason_codes: vec![ReasonCode::DeterministicPath],
    };
    panel.stabilize();
    panel
}

fn build_depth_next_action_panel(
    report: &KISEvidenceDepthReport,
    config_path: Option<&Path>,
) -> NextActionPanel {
    let primary_next_action = NextActionItem {
        action_id: "step-05-kis-evidence-depth-run".to_string(),
        action_kind: NextActionKind::RunControlTowerRefresh,
        priority: NextActionPriority::Required,
        command_suggestion: config_path.map(|path| {
            format!(
                "cargo run --quiet --bin soma_experiment -- control-tower-refresh --config {}",
                path.display()
            )
        }),
        expected_output_artifact: Some("dashboard_state_v1.json".to_string()),
        safe_to_run: true,
        reason_codes: vec![ReasonCode::DeterministicPath],
    };
    let mut panel = NextActionPanel {
        primary_next_action,
        recommended_actions: vec![NextActionItem {
            action_id: "step-01-follow-depth-recommendation".to_string(),
            action_kind: map_recommendation(report.final_recommendation),
            priority: NextActionPriority::Recommended,
            command_suggestion: recommendation_to_command(report.final_recommendation),
            expected_output_artifact: Some(report.primary_bottleneck_after.clone()),
            safe_to_run: true,
            reason_codes: vec![ReasonCode::DeterministicPath],
        }],
        blocked_actions: Vec::new(),
        operator_actions: Vec::new(),
        reason_codes: vec![ReasonCode::DeterministicPath],
    };
    panel.stabilize();
    panel
}

fn map_step_kind(step_kind: OperationalRunbookStepKind) -> NextActionKind {
    match step_kind {
        OperationalRunbookStepKind::KISAuthCheck => NextActionKind::RunKISAuthCheck,
        OperationalRunbookStepKind::KISMarketDataActivate => {
            NextActionKind::RunKISMarketDataActivate
        }
        OperationalRunbookStepKind::KISCandleSufficiency => NextActionKind::RunKISCandleSufficiency,
        OperationalRunbookStepKind::KISOutcomeLinkClose => NextActionKind::RunKISOutcomeLinkClose,
        OperationalRunbookStepKind::KISEvidenceDepthRun => NextActionKind::RunKISEvidenceDepth,
        OperationalRunbookStepKind::TrinityOperationalLoop => NextActionKind::RunCommitteeBenchmark,
        OperationalRunbookStepKind::ControlTowerRefresh => NextActionKind::RunControlTowerRefresh,
        OperationalRunbookStepKind::DashboardOpen => NextActionKind::RunDashboardRender,
        OperationalRunbookStepKind::OwnerReviewQueue => NextActionKind::RunOwnerReviewQueue,
        OperationalRunbookStepKind::CorePerformance => NextActionKind::RunCorePerformance,
        OperationalRunbookStepKind::StopDueToBlocker => NextActionKind::NoAction,
    }
}

fn map_recommendation(recommendation: KISEvidenceDepthFinalRecommendation) -> NextActionKind {
    match recommendation {
        KISEvidenceDepthFinalRecommendation::RunKISCandleSufficiency => {
            NextActionKind::RunKISCandleSufficiency
        }
        KISEvidenceDepthFinalRecommendation::ImproveOutcomeLinkDepth => {
            NextActionKind::ImproveOutcomeLinkDepth
        }
        KISEvidenceDepthFinalRecommendation::ImproveCounterfactualDepth => {
            NextActionKind::RunOperationalRunbook
        }
        KISEvidenceDepthFinalRecommendation::RunDiversitySweep => {
            NextActionKind::ImproveKISEvidence
        }
        KISEvidenceDepthFinalRecommendation::RunCorePerformance => {
            NextActionKind::RunCorePerformance
        }
        KISEvidenceDepthFinalRecommendation::RunTrinityLoop => {
            NextActionKind::RunCommitteeBenchmark
        }
        KISEvidenceDepthFinalRecommendation::RefreshControlTower => {
            NextActionKind::RunControlTowerRefresh
        }
        KISEvidenceDepthFinalRecommendation::KeepTrinity
        | KISEvidenceDepthFinalRecommendation::NeedMoreEvidence => NextActionKind::NoAction,
    }
}

fn recommendation_to_command(
    recommendation: KISEvidenceDepthFinalRecommendation,
) -> Option<String> {
    match recommendation {
        KISEvidenceDepthFinalRecommendation::RunKISCandleSufficiency => Some(
            "cargo run --quiet --bin soma_experiment -- kis-candle-sufficiency --config examples/soma_kis_candle_sufficiency.toml"
                .to_string(),
        ),
        KISEvidenceDepthFinalRecommendation::ImproveOutcomeLinkDepth => Some(
            "cargo run --quiet --bin soma_experiment -- kis-outcome-link-close --config examples/soma_kis_outcome_link_close.toml"
                .to_string(),
        ),
        KISEvidenceDepthFinalRecommendation::ImproveCounterfactualDepth => Some(
            "cargo run --quiet --bin soma_experiment -- operational-runbook --config examples/soma_operational_runbook_kis_loop.toml"
                .to_string(),
        ),
        KISEvidenceDepthFinalRecommendation::RunDiversitySweep => Some(
            "cargo run --quiet --bin soma_experiment -- kis-evidence-depth-run --config examples/soma_kis_evidence_depth_run.toml"
                .to_string(),
        ),
        KISEvidenceDepthFinalRecommendation::RunCorePerformance => Some(
            "cargo run --quiet --bin soma_experiment -- core-performance --config examples/soma_core_performance_diagnostics_only.toml"
                .to_string(),
        ),
        KISEvidenceDepthFinalRecommendation::RunTrinityLoop => Some(
            "cargo run --quiet --bin soma_experiment -- trinity-operational-loop --config examples/soma_trinity_operational_loop_kis.toml"
                .to_string(),
        ),
        KISEvidenceDepthFinalRecommendation::RefreshControlTower => Some(
            "cargo run --quiet --bin soma_experiment -- control-tower-refresh --config examples/soma_control_tower_refresh_after_kis_depth.toml"
                .to_string(),
        ),
        KISEvidenceDepthFinalRecommendation::KeepTrinity
        | KISEvidenceDepthFinalRecommendation::NeedMoreEvidence => None,
    }
}

fn derive_refresh_status(
    render_report: &DashboardV1RenderReport,
    depth_report: Option<&KISEvidenceDepthReport>,
    missing_sources: bool,
) -> ControlTowerRefreshStatus {
    if !render_report.secret_redaction_passed {
        ControlTowerRefreshStatus::SecretRedactionFailed
    } else if render_report.unsafe_control_detected {
        ControlTowerRefreshStatus::UnsafeControlDetected
    } else if missing_sources {
        ControlTowerRefreshStatus::MissingSourceReports
    } else if matches!(
        depth_report.map(|report| report.final_recommendation),
        Some(KISEvidenceDepthFinalRecommendation::NeedMoreEvidence)
    ) {
        ControlTowerRefreshStatus::NoImprovement
    } else if matches!(
        render_report.render_status,
        DashboardV1RenderStatus::Rendered
    ) {
        ControlTowerRefreshStatus::Refreshed
    } else {
        ControlTowerRefreshStatus::RefreshedWithWarnings
    }
}

fn map_operational_timeline(
    timeline: &crate::league::OperationalAuditTimeline,
    loop_id: &str,
) -> AuditTimelinePanel {
    let mut panel = AuditTimelinePanel {
        events: timeline
            .events
            .iter()
            .map(|event| DashboardEvent {
                event_id: event.event_id.clone(),
                kind: match event.event_kind {
                    crate::league::OperationalEventKind::CandidateGenerated
                    | crate::league::OperationalEventKind::CandidateStateChanged => {
                        DashboardEventKind::CandidateStateChange
                    }
                    crate::league::OperationalEventKind::PersonaStartedAnalysis
                    | crate::league::OperationalEventKind::PersonaVoted => {
                        DashboardEventKind::CommitteeVote
                    }
                    crate::league::OperationalEventKind::ChairReviewed => {
                        DashboardEventKind::ChairDecision
                    }
                    crate::league::OperationalEventKind::RiskReviewed
                    | crate::league::OperationalEventKind::RiskBlocked
                    | crate::league::OperationalEventKind::NoTrade => {
                        DashboardEventKind::RiskDecision
                    }
                    crate::league::OperationalEventKind::OwnerReviewQueued => {
                        DashboardEventKind::HumanConfirmStateChange
                    }
                    crate::league::OperationalEventKind::OwnerInputApplied => {
                        DashboardEventKind::OwnerInputApplied
                    }
                    crate::league::OperationalEventKind::PaperApproved => {
                        DashboardEventKind::PaperConfirmed
                    }
                    crate::league::OperationalEventKind::PaperPositionOpened
                    | crate::league::OperationalEventKind::PaperPositionClosed => {
                        DashboardEventKind::PaperPositionStateChange
                    }
                    crate::league::OperationalEventKind::ReanalysisRequested => {
                        DashboardEventKind::ReanalysisRequested
                    }
                    crate::league::OperationalEventKind::Error => DashboardEventKind::Error,
                },
                timestamp_ms: event.timestamp_ms,
                title: format!("{:?}", event.event_kind),
                summary: event.summary.clone(),
                source_report: Some(loop_id.to_string()),
                severity: if matches!(event.event_kind, crate::league::OperationalEventKind::Error)
                {
                    DashboardEventSeverity::Error
                } else if event.reason_codes.len() > 1 {
                    DashboardEventSeverity::Warning
                } else {
                    DashboardEventSeverity::Info
                },
                reason_codes: event.reason_codes.clone(),
            })
            .collect(),
        warnings: 0,
        errors: 0,
        critical_count: 0,
        fingerprint: String::new(),
        reason_codes: vec![ReasonCode::DeterministicPath],
    };
    panel.stabilize();
    panel
}

impl From<PaperPositionPanel> for super::operational_loop_panel::PaperLifecyclePanel {
    fn from(panel: PaperPositionPanel) -> Self {
        Self {
            open_positions: panel.open_positions.len(),
            closed_positions: panel.closed_positions.len(),
            target_hit_count: panel
                .closed_positions
                .iter()
                .filter(|item| matches!(item.status, super::PaperPositionStatus::TargetHit))
                .count(),
            stop_hit_count: panel
                .closed_positions
                .iter()
                .filter(|item| matches!(item.status, super::PaperPositionStatus::Stopped))
                .count(),
            expired_count: panel
                .closed_positions
                .iter()
                .filter(|item| matches!(item.status, super::PaperPositionStatus::Expired))
                .count(),
            risk_closed_count: panel.risk_closed_positions.len(),
            average_unrealized_return: None,
            average_realized_return: None,
            reason_codes: vec![ReasonCode::DeterministicPath],
        }
    }
}

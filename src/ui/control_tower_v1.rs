use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::core::{ReasonCode, stable_hash_string, stable_ordered_strings, stable_reason_codes};
use crate::league::TrinityOperationalLoopReport;

use super::{
    candidate_lifecycle_panel::CandidateLifecyclePanel,
    control_tower_health::{ControlTowerHealthSummary, summarize_control_tower_health},
    core_mamba_readiness_panel::{
        CoreMambaReadinessPanel, build_core_mamba_readiness_panel_from_values,
    },
    dashboard_events::AuditTimelinePanel,
    dashboard_panels::{
        BottleneckPanel, CandidatePanel, ChairPanel, CommitteePanel, EvidencePanel,
        HumanConfirmPanel, PaperPositionPanel, ProviderPanel, RiskPanel,
    },
    dashboard_snapshot::DashboardSnapshotBuilder,
    dashboard_state::{DashboardSourceConfig, DashboardSystemMode},
    kis_monitor_panel::{KISMonitorPanel, build_kis_monitor_panel},
    next_action_panel::{NextActionPanel, build_next_action_panel},
    operational_loop_panel::{OperationalLoopPanel, PaperLifecyclePanel, TrinityStatusPanel},
    owner_panel::OwnerPanel,
};

fn default_true() -> bool {
    true
}

fn default_control_tower_id() -> String {
    "soma_control_tower_v1".to_string()
}

fn default_output_root() -> String {
    "target/sprint54/control_tower_v1".to_string()
}

fn default_max_events() -> usize {
    500
}

fn default_max_candidates() -> usize {
    100
}

fn default_max_owner_inputs() -> usize {
    100
}

fn default_max_paper_positions() -> usize {
    64
}

fn default_max_artifacts() -> usize {
    64
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ControlTowerWatcherMode {
    #[default]
    RefreshPlannerOnly,
    WatcherDeferred,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ControlTowerRefreshPlanner {
    pub watcher_mode: ControlTowerWatcherMode,
    pub tracked_inputs: usize,
    pub missing_outputs: usize,
    #[serde(default)]
    pub refresh_commands: Vec<String>,
    #[serde(default)]
    pub expected_output_artifacts: Vec<String>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl ControlTowerRefreshPlanner {
    pub fn stabilize(&mut self) {
        self.refresh_commands = stable_ordered_strings(&self.refresh_commands);
        self.expected_output_artifacts = stable_ordered_strings(&self.expected_output_artifacts);
        self.reason_codes = stable_reason_codes(&self.reason_codes);
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ControlTowerV1Config {
    #[serde(default = "default_control_tower_id")]
    pub control_tower_id: String,
    #[serde(default)]
    pub dashboard_source_config_paths: Vec<String>,
    #[serde(default)]
    pub provider_simplification_report_paths: Vec<String>,
    #[serde(default)]
    pub kis_activation_report_paths: Vec<String>,
    #[serde(default)]
    pub kis_collection_closure_paths: Vec<String>,
    #[serde(default)]
    pub kis_market_data_activation_paths: Vec<String>,
    #[serde(default)]
    pub owner_review_queue_paths: Vec<String>,
    #[serde(default)]
    pub owner_input_paths: Vec<String>,
    #[serde(default)]
    pub owner_impact_report_paths: Vec<String>,
    #[serde(default)]
    pub owner_thesis_book_paths: Vec<String>,
    #[serde(default)]
    pub committee_benchmark_paths: Vec<String>,
    #[serde(default)]
    pub committee_v1_paths: Vec<String>,
    #[serde(default)]
    pub committee_diagnostics_paths: Vec<String>,
    #[serde(default)]
    pub risk_report_paths: Vec<String>,
    #[serde(default)]
    pub candidate_queue_paths: Vec<String>,
    #[serde(default)]
    pub paper_position_paths: Vec<String>,
    #[serde(default)]
    pub human_confirm_paths: Vec<String>,
    #[serde(default)]
    pub core_scorecard_paths: Vec<String>,
    #[serde(default)]
    pub audit_ledger_paths: Vec<String>,
    #[serde(default)]
    pub core_completion_audit_paths: Vec<String>,
    #[serde(default)]
    pub sequence_readiness_paths: Vec<String>,
    #[serde(default)]
    pub mamba_readiness_v2_paths: Vec<String>,
    #[serde(default)]
    pub model_escalation_decision_paths: Vec<String>,
    #[serde(default)]
    pub operational_loop_report_paths: Vec<String>,
    #[serde(default = "default_output_root")]
    pub output_root: String,
    #[serde(default = "default_max_events")]
    pub max_events: usize,
    #[serde(default = "default_max_candidates")]
    pub max_candidates: usize,
    #[serde(default = "default_max_owner_inputs")]
    pub max_owner_inputs: usize,
    #[serde(default = "default_max_paper_positions")]
    pub max_paper_positions: usize,
    #[serde(default = "default_max_artifacts")]
    pub max_artifacts: usize,
    #[serde(default = "default_true")]
    pub render_html: bool,
    #[serde(default = "default_true")]
    pub render_json: bool,
    #[serde(default = "default_true")]
    pub render_text: bool,
    #[serde(default = "default_true")]
    pub generate_owner_action_drafts: bool,
    #[serde(default)]
    pub enable_dashboard_open: bool,
    #[serde(default)]
    pub enable_dashboard_serve: bool,
    #[serde(default = "default_true")]
    pub redact_secrets: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl Default for ControlTowerV1Config {
    fn default() -> Self {
        Self {
            control_tower_id: default_control_tower_id(),
            dashboard_source_config_paths: Vec::new(),
            provider_simplification_report_paths: Vec::new(),
            kis_activation_report_paths: Vec::new(),
            kis_collection_closure_paths: Vec::new(),
            kis_market_data_activation_paths: Vec::new(),
            owner_review_queue_paths: Vec::new(),
            owner_input_paths: Vec::new(),
            owner_impact_report_paths: Vec::new(),
            owner_thesis_book_paths: Vec::new(),
            committee_benchmark_paths: Vec::new(),
            committee_v1_paths: Vec::new(),
            committee_diagnostics_paths: Vec::new(),
            risk_report_paths: Vec::new(),
            candidate_queue_paths: Vec::new(),
            paper_position_paths: Vec::new(),
            human_confirm_paths: Vec::new(),
            core_scorecard_paths: Vec::new(),
            audit_ledger_paths: Vec::new(),
            core_completion_audit_paths: Vec::new(),
            sequence_readiness_paths: Vec::new(),
            mamba_readiness_v2_paths: Vec::new(),
            model_escalation_decision_paths: Vec::new(),
            operational_loop_report_paths: Vec::new(),
            output_root: default_output_root(),
            max_events: default_max_events(),
            max_candidates: default_max_candidates(),
            max_owner_inputs: default_max_owner_inputs(),
            max_paper_positions: default_max_paper_positions(),
            max_artifacts: default_max_artifacts(),
            render_html: true,
            render_json: true,
            render_text: true,
            generate_owner_action_drafts: true,
            enable_dashboard_open: false,
            enable_dashboard_serve: false,
            redact_secrets: true,
            reason_codes: vec![ReasonCode::DeterministicPath],
        }
    }
}

impl ControlTowerV1Config {
    pub fn from_toml_str(input: &str) -> Result<Self, String> {
        toml::from_str(input).map_err(|err| err.to_string())
    }

    pub fn from_toml_path(path: &Path) -> Result<Self, String> {
        let contents = fs::read_to_string(path).map_err(|err| err.to_string())?;
        Self::from_toml_str(&contents)
    }

    pub fn to_toml_string(&self) -> Result<String, String> {
        toml::to_string_pretty(self).map_err(|err| err.to_string())
    }

    pub fn validate_local_paths(&self) -> Vec<ReasonCode> {
        if self
            .all_input_paths()
            .iter()
            .chain([self.output_root.clone()].iter())
            .any(|path| path.contains("://"))
        {
            vec![
                ReasonCode::LocalPathRejected,
                ReasonCode::RemotePathRejected,
            ]
        } else {
            Vec::new()
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.control_tower_id.trim().is_empty() {
            return Err("control tower id must not be empty".to_string());
        }
        if !self.validate_local_paths().is_empty() {
            return Err("control-tower-v1 config paths must be local".to_string());
        }
        if self.max_events > 1000 {
            return Err("control-tower-v1 max_events must be <= 1000".to_string());
        }
        if self.max_candidates > 200 {
            return Err("control-tower-v1 max_candidates must be <= 200".to_string());
        }
        if self.max_owner_inputs > 200 {
            return Err("control-tower-v1 max_owner_inputs must be <= 200".to_string());
        }
        if self.max_paper_positions > 100 {
            return Err("control-tower-v1 max_paper_positions must be <= 100".to_string());
        }
        if self.max_artifacts > 256 {
            return Err("control-tower-v1 max_artifacts must be <= 256".to_string());
        }
        Ok(())
    }

    pub fn artifact_dir(&self) -> PathBuf {
        Path::new(&self.output_root).join(&self.control_tower_id)
    }

    pub fn all_input_paths(&self) -> Vec<String> {
        stable_ordered_strings(
            &self
                .dashboard_source_config_paths
                .iter()
                .chain(self.provider_simplification_report_paths.iter())
                .chain(self.kis_activation_report_paths.iter())
                .chain(self.kis_collection_closure_paths.iter())
                .chain(self.kis_market_data_activation_paths.iter())
                .chain(self.owner_review_queue_paths.iter())
                .chain(self.owner_input_paths.iter())
                .chain(self.owner_impact_report_paths.iter())
                .chain(self.owner_thesis_book_paths.iter())
                .chain(self.committee_benchmark_paths.iter())
                .chain(self.committee_v1_paths.iter())
                .chain(self.committee_diagnostics_paths.iter())
                .chain(self.risk_report_paths.iter())
                .chain(self.candidate_queue_paths.iter())
                .chain(self.paper_position_paths.iter())
                .chain(self.human_confirm_paths.iter())
                .chain(self.core_scorecard_paths.iter())
                .chain(self.audit_ledger_paths.iter())
                .chain(self.core_completion_audit_paths.iter())
                .chain(self.sequence_readiness_paths.iter())
                .chain(self.mamba_readiness_v2_paths.iter())
                .chain(self.model_escalation_decision_paths.iter())
                .chain(self.operational_loop_report_paths.iter())
                .cloned()
                .collect::<Vec<_>>(),
        )
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ControlTowerV1State {
    pub control_tower_id: String,
    #[serde(default)]
    pub generated_from_reports: Vec<String>,
    pub system_mode: DashboardSystemMode,
    pub provider_panel: ProviderPanel,
    pub kis_monitor_panel: KISMonitorPanel,
    pub evidence_panel: EvidencePanel,
    pub committee_panel: CommitteePanel,
    pub chair_panel: ChairPanel,
    pub risk_panel: RiskPanel,
    pub candidate_panel: CandidatePanel,
    pub paper_position_panel: PaperPositionPanel,
    pub operational_loop_panel: OperationalLoopPanel,
    pub trinity_status_panel: TrinityStatusPanel,
    pub paper_lifecycle_panel: PaperLifecyclePanel,
    pub candidate_lifecycle_panel: CandidateLifecyclePanel,
    pub owner_panel: OwnerPanel,
    pub human_confirm_panel: HumanConfirmPanel,
    pub bottleneck_panel: BottleneckPanel,
    pub next_action_panel: NextActionPanel,
    pub audit_timeline: AuditTimelinePanel,
    pub health_summary: ControlTowerHealthSummary,
    pub refresh_planner: ControlTowerRefreshPlanner,
    #[serde(default)]
    pub core_mamba_readiness_panel: Option<CoreMambaReadinessPanel>,
    #[serde(default)]
    pub warnings: Vec<String>,
    #[serde(default)]
    pub blockers: Vec<String>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
    pub fingerprint: String,
}

impl ControlTowerV1State {
    pub fn with_fingerprint(mut self) -> Self {
        self.provider_panel.stabilize();
        self.kis_monitor_panel.stabilize();
        self.evidence_panel.stabilize();
        self.committee_panel.stabilize();
        self.chair_panel.stabilize();
        self.risk_panel.stabilize();
        self.candidate_panel.stabilize();
        self.paper_position_panel.stabilize();
        self.operational_loop_panel.stabilize();
        self.trinity_status_panel.stabilize();
        self.paper_lifecycle_panel.stabilize();
        self.candidate_lifecycle_panel.stabilize();
        self.owner_panel.stabilize();
        self.human_confirm_panel.stabilize();
        self.bottleneck_panel.stabilize();
        self.next_action_panel.stabilize();
        self.audit_timeline.stabilize();
        self.health_summary.stabilize();
        self.refresh_planner.stabilize();
        if let Some(panel) = &mut self.core_mamba_readiness_panel {
            panel.stabilize();
        }
        self.generated_from_reports = stable_ordered_strings(&self.generated_from_reports);
        self.warnings = stable_ordered_strings(&self.warnings);
        self.blockers = stable_ordered_strings(&self.blockers);
        self.reason_codes = stable_reason_codes(&self.reason_codes);
        self.fingerprint = String::new();
        let material = serde_json::to_string(&self).unwrap_or_else(|_| self.to_text());
        self.fingerprint = stable_hash_string(&material);
        self
    }

    pub fn to_json_string(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|err| err.to_string())
    }

    pub fn to_text(&self) -> String {
        [
            "read_only_warning=control tower v1 is local-only deterministic monitoring and owner-draft generation only".to_string(),
            "paper_only_warning=no live trading broker order account balance holdings position or execution controls exist".to_string(),
            format!("control_tower_id={}", self.control_tower_id),
            format!("system_mode={:?}", self.system_mode),
            format!("generated_from_reports={}", self.generated_from_reports.join("|")),
            format!("kis_auth_ready={}", self.kis_monitor_panel.auth_ready),
            format!("kis_base_url_ready={}", self.kis_monitor_panel.base_url_ready),
            format!("kis_endpoint_policy_status={}", self.kis_monitor_panel.endpoint_policy_status),
            format!("kis_collection_plan_status={}", self.kis_monitor_panel.collection_plan_status),
            format!("owner_pending_reviews={}", self.owner_panel.pending_review_items.len()),
            format!("risk_denied_count={}", self.risk_panel.denied_count),
            format!("candidate_count={}", self.candidate_panel.candidates.len()),
            format!("paper_open_positions={}", self.paper_position_panel.open_positions.len()),
            format!("operational_loop_status={}", self.operational_loop_panel.loop_status),
            format!(
                "candidate_lifecycle_count={}",
                self.candidate_lifecycle_panel.candidate_views.len()
            ),
            format!(
                "trinity_active_personas={}",
                self.trinity_status_panel.active_count
            ),
            format!(
                "paper_lifecycle_open={}",
                self.paper_lifecycle_panel.open_positions
            ),
            format!(
                "primary_next_action={:?}",
                self.next_action_panel.primary_next_action.action_kind
            ),
            format!("health_status={:?}", self.health_summary.health_status),
            format!(
                "core_mamba_panel={}",
                self.core_mamba_readiness_panel
                    .as_ref()
                    .map(|panel| panel.to_text().replace("\n", " || "))
                    .unwrap_or_else(|| "Unavailable".to_string())
            ),
            format!("fingerprint={}", self.fingerprint),
        ]
        .join("\n")
    }
}

#[derive(Clone, Debug, Default)]
pub struct ControlTowerV1Builder;

impl ControlTowerV1Builder {
    pub fn build(
        &self,
        config: &ControlTowerV1Config,
        config_path: Option<&Path>,
    ) -> Result<ControlTowerV1State, String> {
        config.validate()?;
        let mut warnings = Vec::new();
        let mut blockers = Vec::new();
        let mut reason_codes = config.reason_codes.clone();
        reason_codes.push(ReasonCode::DashboardStateBuilt);

        let (source_config, source_config_paths) = merged_dashboard_source_config(config)?;
        let base_state = DashboardSnapshotBuilder::default().build(&source_config)?;
        warnings.extend(base_state.warnings.clone());
        blockers.extend(base_state.blockers.clone());
        reason_codes.extend(base_state.reason_codes.clone());

        let kis_activation_values = load_values(
            &config.kis_activation_report_paths,
            "kis activation",
            &mut warnings,
            &mut reason_codes,
        );
        let kis_collection_values = load_values(
            &config.kis_collection_closure_paths,
            "kis collection closure",
            &mut warnings,
            &mut reason_codes,
        );
        let kis_market_values = load_values(
            &config.kis_market_data_activation_paths,
            "kis market data activation",
            &mut warnings,
            &mut reason_codes,
        );
        let core_completion_values = load_values(
            &config.core_completion_audit_paths,
            "core completion audit",
            &mut warnings,
            &mut reason_codes,
        );
        let sequence_readiness_values = load_values(
            &config.sequence_readiness_paths,
            "sequence readiness",
            &mut warnings,
            &mut reason_codes,
        );
        let mamba_readiness_values = load_values(
            &config.mamba_readiness_v2_paths,
            "mamba readiness v2",
            &mut warnings,
            &mut reason_codes,
        );
        let model_escalation_values = load_values(
            &config.model_escalation_decision_paths,
            "model escalation decision",
            &mut warnings,
            &mut reason_codes,
        );

        let kis_monitor_panel = build_kis_monitor_panel(
            &base_state.provider_panel,
            &base_state.evidence_panel,
            &kis_activation_values,
            &kis_collection_values,
            &kis_market_values,
        );

        let core_mamba_readiness_panel = build_core_mamba_readiness_panel_from_values(
            &core_completion_values,
            &mamba_readiness_values,
            &sequence_readiness_values,
            &model_escalation_values,
        );

        let refresh_planner = build_refresh_planner(config, config_path);
        let loop_report = load_latest_operational_loop_report(
            &config.operational_loop_report_paths,
            &mut warnings,
            &mut reason_codes,
        );
        let operational_loop_panel = loop_report
            .as_ref()
            .map(|report| report.operational_loop_panel.clone())
            .unwrap_or_default();
        let trinity_status_panel = loop_report
            .as_ref()
            .map(|report| report.trinity_status_panel.clone())
            .unwrap_or_default();
        let paper_lifecycle_panel = loop_report
            .as_ref()
            .map(|report| report.paper_lifecycle_panel.clone())
            .unwrap_or_default();
        let candidate_lifecycle_panel = loop_report
            .as_ref()
            .map(|report| report.candidate_lifecycle_panel.clone())
            .unwrap_or_default();

        let next_action_panel = build_next_action_panel(
            config_path
                .map(|path| path.display().to_string())
                .as_deref(),
            &config.output_root,
            &kis_monitor_panel,
            &base_state.owner_panel,
            &base_state.risk_panel,
            &base_state.candidate_panel,
            &refresh_planner,
        );

        let health_summary = summarize_control_tower_health(
            &kis_monitor_panel,
            &base_state.evidence_panel,
            &base_state.owner_panel,
            &base_state.risk_panel,
            &base_state.candidate_panel,
            &next_action_panel,
            &refresh_planner,
            &base_state.warnings,
            &base_state.blockers,
            config.redact_secrets,
        );

        if !kis_monitor_panel.auth_ready {
            blockers.push("KIS auth not ready".to_string());
        }
        if !kis_monitor_panel.base_url_ready {
            blockers.push("KIS base URL not ready".to_string());
        }
        if next_action_panel.primary_next_action.safe_to_run {
            warnings.push("Control Tower v1 remains read-only and local-only.".to_string());
        }

        let generated_from_reports = stable_ordered_strings(
            &base_state
                .generated_from_reports
                .iter()
                .chain(source_config_paths.iter())
                .chain(config.kis_collection_closure_paths.iter())
                .chain(config.kis_market_data_activation_paths.iter())
                .chain(config.core_completion_audit_paths.iter())
                .chain(config.sequence_readiness_paths.iter())
                .chain(config.mamba_readiness_v2_paths.iter())
                .chain(config.model_escalation_decision_paths.iter())
                .chain(config.operational_loop_report_paths.iter())
                .cloned()
                .collect::<Vec<_>>(),
        )
        .into_iter()
        .take(config.max_artifacts)
        .collect::<Vec<_>>();

        let mut state = ControlTowerV1State {
            control_tower_id: config.control_tower_id.clone(),
            generated_from_reports,
            system_mode: base_state.system_mode,
            provider_panel: base_state.provider_panel,
            kis_monitor_panel,
            evidence_panel: base_state.evidence_panel,
            committee_panel: base_state.committee_panel,
            chair_panel: base_state.chair_panel,
            risk_panel: base_state.risk_panel,
            candidate_panel: truncate_candidates(base_state.candidate_panel, config.max_candidates),
            paper_position_panel: truncate_paper_positions(
                base_state.paper_position_panel,
                config.max_paper_positions,
            ),
            operational_loop_panel,
            trinity_status_panel,
            paper_lifecycle_panel,
            candidate_lifecycle_panel,
            owner_panel: truncate_owner_panel(base_state.owner_panel, config.max_owner_inputs),
            human_confirm_panel: base_state.human_confirm_panel,
            bottleneck_panel: base_state.bottleneck_panel,
            next_action_panel,
            audit_timeline: base_state.audit_timeline,
            health_summary,
            refresh_planner,
            core_mamba_readiness_panel,
            warnings,
            blockers,
            reason_codes: stable_reason_codes(&reason_codes),
            fingerprint: String::new(),
        }
        .with_fingerprint();

        if config.redact_secrets {
            state = redact_state_strings(&state)?;
        }

        Ok(state.with_fingerprint())
    }
}

fn merged_dashboard_source_config(
    config: &ControlTowerV1Config,
) -> Result<(DashboardSourceConfig, Vec<String>), String> {
    let mut source = DashboardSourceConfig {
        dashboard_id: config.control_tower_id.clone(),
        output_root: config.output_root.clone(),
        max_events: config.max_events,
        max_candidates: config.max_candidates,
        max_committee_rows: 12,
        max_artifacts: config.max_artifacts,
        include_diagnostics: true,
        include_research_only: true,
        include_fixture_only: true,
        include_crypto_only: false,
        redact_secrets: config.redact_secrets,
        reason_codes: stable_reason_codes(&config.reason_codes),
        ..DashboardSourceConfig::default()
    };
    let mut loaded_source_configs = Vec::new();
    for path in &config.dashboard_source_config_paths {
        let loaded = DashboardSourceConfig::from_toml_path(Path::new(path))?;
        loaded_source_configs.push(path.clone());
        source
            .provider_simplification_report_paths
            .extend(loaded.provider_simplification_report_paths);
        source
            .kis_activation_report_paths
            .extend(loaded.kis_activation_report_paths);
        source
            .kis_collection_closure_paths
            .extend(loaded.kis_collection_closure_paths);
        source
            .krx_activation_report_paths
            .extend(loaded.krx_activation_report_paths);
        source
            .official_evidence_scaleout_paths
            .extend(loaded.official_evidence_scaleout_paths);
        source
            .official_evidence_diversity_paths
            .extend(loaded.official_evidence_diversity_paths);
        source
            .core_performance_scorecard_paths
            .extend(loaded.core_performance_scorecard_paths);
        source
            .committee_benchmark_paths
            .extend(loaded.committee_benchmark_paths);
        source.committee_v1_paths.extend(loaded.committee_v1_paths);
        source
            .committee_diagnostics_paths
            .extend(loaded.committee_diagnostics_paths);
        source.risk_reports.extend(loaded.risk_reports);
        source.audit_ledger_paths.extend(loaded.audit_ledger_paths);
        source
            .candidate_queue_paths
            .extend(loaded.candidate_queue_paths);
        source
            .paper_position_paths
            .extend(loaded.paper_position_paths);
        source
            .human_confirm_paths
            .extend(loaded.human_confirm_paths);
        source.owner_input_paths.extend(loaded.owner_input_paths);
        source
            .owner_thesis_note_paths
            .extend(loaded.owner_thesis_note_paths);
        source
            .owner_review_queue_paths
            .extend(loaded.owner_review_queue_paths);
        source
            .owner_impact_report_paths
            .extend(loaded.owner_impact_report_paths);
    }

    source
        .provider_simplification_report_paths
        .extend(config.provider_simplification_report_paths.clone());
    source
        .kis_activation_report_paths
        .extend(config.kis_activation_report_paths.clone());
    source
        .kis_collection_closure_paths
        .extend(config.kis_collection_closure_paths.clone());
    source
        .core_performance_scorecard_paths
        .extend(config.core_scorecard_paths.clone());
    source
        .committee_benchmark_paths
        .extend(config.committee_benchmark_paths.clone());
    source
        .committee_v1_paths
        .extend(config.committee_v1_paths.clone());
    source
        .committee_diagnostics_paths
        .extend(config.committee_diagnostics_paths.clone());
    source.risk_reports.extend(config.risk_report_paths.clone());
    source
        .candidate_queue_paths
        .extend(config.candidate_queue_paths.clone());
    source
        .paper_position_paths
        .extend(config.paper_position_paths.clone());
    source
        .human_confirm_paths
        .extend(config.human_confirm_paths.clone());
    source
        .owner_input_paths
        .extend(config.owner_input_paths.clone());
    source
        .owner_thesis_note_paths
        .extend(config.owner_thesis_book_paths.clone());
    source
        .owner_review_queue_paths
        .extend(config.owner_review_queue_paths.clone());
    source
        .owner_impact_report_paths
        .extend(config.owner_impact_report_paths.clone());
    source
        .audit_ledger_paths
        .extend(config.audit_ledger_paths.clone());

    source.provider_simplification_report_paths =
        stable_ordered_strings(&source.provider_simplification_report_paths);
    source.kis_activation_report_paths =
        stable_ordered_strings(&source.kis_activation_report_paths);
    source.kis_collection_closure_paths =
        stable_ordered_strings(&source.kis_collection_closure_paths);
    source.core_performance_scorecard_paths =
        stable_ordered_strings(&source.core_performance_scorecard_paths);
    source.committee_benchmark_paths = stable_ordered_strings(&source.committee_benchmark_paths);
    source.committee_v1_paths = stable_ordered_strings(&source.committee_v1_paths);
    source.committee_diagnostics_paths =
        stable_ordered_strings(&source.committee_diagnostics_paths);
    source.risk_reports = stable_ordered_strings(&source.risk_reports);
    source.candidate_queue_paths = stable_ordered_strings(&source.candidate_queue_paths);
    source.paper_position_paths = stable_ordered_strings(&source.paper_position_paths);
    source.human_confirm_paths = stable_ordered_strings(&source.human_confirm_paths);
    source.owner_input_paths = stable_ordered_strings(&source.owner_input_paths);
    source.owner_thesis_note_paths = stable_ordered_strings(&source.owner_thesis_note_paths);
    source.owner_review_queue_paths = stable_ordered_strings(&source.owner_review_queue_paths);
    source.owner_impact_report_paths = stable_ordered_strings(&source.owner_impact_report_paths);
    source.audit_ledger_paths = stable_ordered_strings(&source.audit_ledger_paths);
    source.reason_codes = stable_reason_codes(&source.reason_codes);

    Ok((source, loaded_source_configs))
}

fn build_refresh_planner(
    config: &ControlTowerV1Config,
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
    let mut refresh_commands = vec![format!(
        "cargo run --quiet --bin soma_experiment -- control-tower-v1 --config {}",
        config_path
            .map(|path| path.display().to_string())
            .unwrap_or_else(|| "examples/soma_control_tower_v1_kis.toml".to_string())
    )];
    refresh_commands.push(format!(
        "cargo run --quiet --bin soma_experiment -- dashboard-action-drafts --config {}",
        config_path
            .map(|path| path.display().to_string())
            .unwrap_or_else(|| "examples/soma_dashboard_action_drafts.toml".to_string())
    ));
    let mut planner = ControlTowerRefreshPlanner {
        watcher_mode: ControlTowerWatcherMode::RefreshPlannerOnly,
        tracked_inputs: config.all_input_paths().len(),
        missing_outputs,
        refresh_commands,
        expected_output_artifacts,
        reason_codes: vec![ReasonCode::DeterministicPath],
    };
    planner.stabilize();
    planner
}

fn load_values(
    paths: &[String],
    label: &str,
    warnings: &mut Vec<String>,
    reason_codes: &mut Vec<ReasonCode>,
) -> Vec<Value> {
    let mut values = Vec::new();
    for path in stable_ordered_strings(paths) {
        match fs::read_to_string(&path) {
            Ok(text) => match serde_json::from_str::<Value>(&text) {
                Ok(value) => values.push(value),
                Err(err) => {
                    warnings.push(format!("failed to parse {label} report: {err}"));
                    reason_codes.push(ReasonCode::DataLoadFailed);
                }
            },
            Err(_) => {
                warnings.push(format!("missing {label} report"));
                reason_codes.push(ReasonCode::MissingFile);
                reason_codes.push(ReasonCode::DashboardReportMissing);
            }
        }
    }
    values
}

fn load_latest_operational_loop_report(
    paths: &[String],
    warnings: &mut Vec<String>,
    reason_codes: &mut Vec<ReasonCode>,
) -> Option<TrinityOperationalLoopReport> {
    let mut reports = Vec::new();
    for path in stable_ordered_strings(paths) {
        match fs::read_to_string(&path) {
            Ok(text) => match serde_json::from_str::<TrinityOperationalLoopReport>(&text) {
                Ok(report) => reports.push(report),
                Err(err) => {
                    warnings.push(format!(
                        "failed to parse operational loop report {}: {}",
                        path, err
                    ));
                    reason_codes.push(ReasonCode::DataLoadFailed);
                }
            },
            Err(_) => {
                warnings.push("missing operational loop report".to_string());
                reason_codes.push(ReasonCode::MissingFile);
            }
        }
    }
    reports.sort_by(|left, right| left.loop_id.cmp(&right.loop_id));
    reports.into_iter().last()
}

fn truncate_candidates(mut panel: CandidatePanel, max_candidates: usize) -> CandidatePanel {
    panel.candidates.truncate(max_candidates);
    panel.stabilize();
    panel
}

fn truncate_paper_positions(
    mut panel: PaperPositionPanel,
    max_paper_positions: usize,
) -> PaperPositionPanel {
    for positions in [
        &mut panel.open_positions,
        &mut panel.closed_positions,
        &mut panel.risk_closed_positions,
        &mut panel.diagnostic_positions,
    ] {
        positions.truncate(max_paper_positions);
    }
    panel.stabilize();
    panel
}

fn truncate_owner_panel(mut panel: OwnerPanel, max_owner_inputs: usize) -> OwnerPanel {
    panel.recent_owner_inputs.truncate(max_owner_inputs);
    panel.blocked_owner_inputs.truncate(max_owner_inputs);
    panel.reanalysis_requests.truncate(max_owner_inputs);
    panel.pending_review_items.truncate(max_owner_inputs);
    panel.paper_confirmed_items.truncate(max_owner_inputs);
    panel.active_thesis_notes.truncate(max_owner_inputs);
    panel.stabilize();
    panel
}

fn redact_state_strings(state: &ControlTowerV1State) -> Result<ControlTowerV1State, String> {
    let mut value = serde_json::to_value(state).map_err(|err| err.to_string())?;
    redact_value(&mut value);
    let mut redacted: ControlTowerV1State =
        serde_json::from_value(value).map_err(|err| err.to_string())?;
    redacted = redacted.with_fingerprint();
    Ok(redacted)
}

fn redact_value(value: &mut Value) {
    match value {
        Value::Object(map) => {
            let keys = map.keys().cloned().collect::<Vec<_>>();
            for key in keys {
                if let Some(child) = map.get_mut(&key) {
                    if is_sensitive_key(&key) && child.is_string() {
                        *child = Value::String("[REDACTED]".to_string());
                    } else {
                        redact_value(child);
                    }
                }
            }
        }
        Value::Array(items) => {
            for item in items {
                redact_value(item);
            }
        }
        Value::String(text) => {
            if looks_sensitive_string(text) {
                *text = "[REDACTED]".to_string();
            }
        }
        _ => {}
    }
}

fn is_sensitive_key(key: &str) -> bool {
    let key = key.to_ascii_lowercase();
    matches!(
        key.as_str(),
        "kis_app_key"
            | "app_key"
            | "kis_app_secret"
            | "app_secret"
            | "kis_ws_approval_key"
            | "krx_api_key"
            | "base_url_preview_redacted"
            | "token"
            | "password"
    )
}

fn looks_sensitive_string(text: &str) -> bool {
    let normalized = text.to_ascii_lowercase();
    [
        "kis_app_key",
        "kis_app_secret",
        "kis_ws_approval_key",
        "krx_api_key",
        "token=",
        "password=",
        "secret=",
        "app_key=",
        "app_secret=",
        "kis_base_url=",
    ]
    .iter()
    .any(|needle| normalized.contains(needle))
}

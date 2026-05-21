use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_hash_string, stable_ordered_strings, stable_reason_codes};

use super::dashboard_events::AuditTimelinePanel;
use super::dashboard_panels::{
    BottleneckPanel, CandidatePanel, ChairPanel, CommitteePanel, EvidencePanel, HumanConfirmPanel,
    PaperPositionPanel, ProviderPanel, RiskPanel,
};
use super::owner_panel::OwnerPanel;

fn default_true() -> bool {
    true
}

fn default_max_events() -> usize {
    500
}

fn default_max_candidates() -> usize {
    100
}

fn default_max_committee_rows() -> usize {
    12
}

fn default_max_artifacts() -> usize {
    32
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum DashboardSystemMode {
    #[default]
    Research,
    Backtest,
    Paper,
    DiagnosticsOnly,
    LiveDisabled,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum DashboardEntityStatus {
    #[default]
    Idle,
    WaitingData,
    CollectingData,
    Preflight,
    Analyzing,
    Voting,
    ChairReview,
    RiskReview,
    Candidate,
    HumanConfirmRequired,
    PaperApproved,
    PaperPositionOpen,
    PaperClosed,
    RiskBlocked,
    NoTrade,
    DiagnosticOnly,
    ResearchOnly,
    Error,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DashboardSourceConfig {
    pub dashboard_id: String,
    #[serde(default)]
    pub provider_simplification_report_paths: Vec<String>,
    #[serde(default)]
    pub kis_activation_report_paths: Vec<String>,
    #[serde(default)]
    pub kis_collection_closure_paths: Vec<String>,
    #[serde(default)]
    pub krx_activation_report_paths: Vec<String>,
    #[serde(default)]
    pub official_evidence_scaleout_paths: Vec<String>,
    #[serde(default)]
    pub official_evidence_diversity_paths: Vec<String>,
    #[serde(default)]
    pub core_performance_scorecard_paths: Vec<String>,
    #[serde(default)]
    pub committee_benchmark_paths: Vec<String>,
    #[serde(default)]
    pub committee_v1_paths: Vec<String>,
    #[serde(default)]
    pub committee_diagnostics_paths: Vec<String>,
    #[serde(default)]
    pub risk_reports: Vec<String>,
    #[serde(default)]
    pub audit_ledger_paths: Vec<String>,
    #[serde(default)]
    pub candidate_queue_paths: Vec<String>,
    #[serde(default)]
    pub paper_position_paths: Vec<String>,
    #[serde(default)]
    pub human_confirm_paths: Vec<String>,
    #[serde(default)]
    pub owner_input_paths: Vec<String>,
    #[serde(default)]
    pub owner_thesis_note_paths: Vec<String>,
    #[serde(default)]
    pub owner_review_queue_paths: Vec<String>,
    #[serde(default)]
    pub owner_impact_report_paths: Vec<String>,
    pub output_root: String,
    #[serde(default = "default_max_events")]
    pub max_events: usize,
    #[serde(default = "default_max_candidates")]
    pub max_candidates: usize,
    #[serde(default = "default_max_committee_rows")]
    pub max_committee_rows: usize,
    #[serde(default = "default_max_artifacts")]
    pub max_artifacts: usize,
    #[serde(default)]
    pub include_diagnostics: bool,
    #[serde(default)]
    pub include_research_only: bool,
    #[serde(default)]
    pub include_fixture_only: bool,
    #[serde(default)]
    pub include_crypto_only: bool,
    #[serde(default = "default_true")]
    pub redact_secrets: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl Default for DashboardSourceConfig {
    fn default() -> Self {
        Self {
            dashboard_id: "soma_control_tower".to_string(),
            provider_simplification_report_paths: Vec::new(),
            kis_activation_report_paths: Vec::new(),
            kis_collection_closure_paths: Vec::new(),
            krx_activation_report_paths: Vec::new(),
            official_evidence_scaleout_paths: Vec::new(),
            official_evidence_diversity_paths: Vec::new(),
            core_performance_scorecard_paths: Vec::new(),
            committee_benchmark_paths: Vec::new(),
            committee_v1_paths: Vec::new(),
            committee_diagnostics_paths: Vec::new(),
            risk_reports: Vec::new(),
            audit_ledger_paths: Vec::new(),
            candidate_queue_paths: Vec::new(),
            paper_position_paths: Vec::new(),
            human_confirm_paths: Vec::new(),
            owner_input_paths: Vec::new(),
            owner_thesis_note_paths: Vec::new(),
            owner_review_queue_paths: Vec::new(),
            owner_impact_report_paths: Vec::new(),
            output_root: "target/soma_control_tower".to_string(),
            max_events: default_max_events(),
            max_candidates: default_max_candidates(),
            max_committee_rows: default_max_committee_rows(),
            max_artifacts: default_max_artifacts(),
            include_diagnostics: false,
            include_research_only: false,
            include_fixture_only: false,
            include_crypto_only: false,
            redact_secrets: true,
            reason_codes: vec![ReasonCode::DeterministicPath],
        }
    }
}

impl DashboardSourceConfig {
    pub fn from_toml_path(path: &Path) -> Result<Self, String> {
        let contents = fs::read_to_string(path).map_err(|err| err.to_string())?;
        toml::from_str(&contents).map_err(|err| err.to_string())
    }

    pub fn to_toml_string(&self) -> Result<String, String> {
        toml::to_string_pretty(self).map_err(|err| err.to_string())
    }

    pub fn validate_local_paths(&self) -> Vec<ReasonCode> {
        let remote = self.output_root.contains("://")
            || self
                .all_report_paths()
                .iter()
                .any(|path| path.contains("://"));
        if remote {
            vec![
                ReasonCode::LocalPathRejected,
                ReasonCode::RemotePathRejected,
            ]
        } else {
            Vec::new()
        }
    }

    pub fn artifact_dir(&self) -> PathBuf {
        Path::new(&self.output_root).join(&self.dashboard_id)
    }

    pub fn all_report_paths(&self) -> Vec<String> {
        stable_ordered_strings(
            &self
                .provider_simplification_report_paths
                .iter()
                .chain(self.kis_activation_report_paths.iter())
                .chain(self.kis_collection_closure_paths.iter())
                .chain(self.krx_activation_report_paths.iter())
                .chain(self.official_evidence_scaleout_paths.iter())
                .chain(self.official_evidence_diversity_paths.iter())
                .chain(self.core_performance_scorecard_paths.iter())
                .chain(self.committee_benchmark_paths.iter())
                .chain(self.committee_v1_paths.iter())
                .chain(self.committee_diagnostics_paths.iter())
                .chain(self.risk_reports.iter())
                .chain(self.audit_ledger_paths.iter())
                .chain(self.candidate_queue_paths.iter())
                .chain(self.paper_position_paths.iter())
                .chain(self.human_confirm_paths.iter())
                .chain(self.owner_input_paths.iter())
                .chain(self.owner_thesis_note_paths.iter())
                .chain(self.owner_review_queue_paths.iter())
                .chain(self.owner_impact_report_paths.iter())
                .cloned()
                .collect::<Vec<_>>(),
        )
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct DashboardState {
    pub dashboard_id: String,
    #[serde(default)]
    pub generated_from_reports: Vec<String>,
    pub system_mode: DashboardSystemMode,
    pub provider_panel: ProviderPanel,
    pub evidence_panel: EvidencePanel,
    pub committee_panel: CommitteePanel,
    pub chair_panel: ChairPanel,
    pub risk_panel: RiskPanel,
    pub candidate_panel: CandidatePanel,
    pub paper_position_panel: PaperPositionPanel,
    pub human_confirm_panel: HumanConfirmPanel,
    pub owner_panel: OwnerPanel,
    pub bottleneck_panel: BottleneckPanel,
    pub audit_timeline: AuditTimelinePanel,
    #[serde(default)]
    pub warnings: Vec<String>,
    #[serde(default)]
    pub blockers: Vec<String>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
    pub fingerprint: String,
}

impl DashboardState {
    pub fn with_fingerprint(mut self) -> Self {
        self.provider_panel.stabilize();
        self.evidence_panel.stabilize();
        self.committee_panel.stabilize();
        self.chair_panel.stabilize();
        self.risk_panel.stabilize();
        self.candidate_panel.stabilize();
        self.paper_position_panel.stabilize();
        self.human_confirm_panel.stabilize();
        self.owner_panel.stabilize();
        self.bottleneck_panel.stabilize();
        self.audit_timeline.stabilize();
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
            "read_only_warning=soma control tower ui is read-only and local-only".to_string(),
            "safety_warning=no broker order account balance holdings position or live execution controls are present"
                .to_string(),
            format!("dashboard_id={}", self.dashboard_id),
            format!("system_mode={:?}", self.system_mode),
            format!("generated_from_reports={}", self.generated_from_reports.join("|")),
            format!(
                "provider_markets={}",
                self.provider_panel
                    .active_primary_provider_by_market
                    .iter()
                    .map(|(market, provider)| format!("{market}:{provider}"))
                    .collect::<Vec<_>>()
                    .join("|")
            ),
            format!("official_rows={}", self.evidence_panel.official_rows),
            format!("candidates={}", self.candidate_panel.candidates.len()),
            format!("paper_open_positions={}", self.paper_position_panel.open_positions.len()),
            format!("human_confirm_pending={}", self.human_confirm_panel.pending_items.len()),
            format!(
                "owner_pending_review={}",
                self.owner_panel.pending_review_items.len()
            ),
            format!("audit_events={}", self.audit_timeline.events.len()),
            format!("warnings={}", self.warnings.join("|")),
            format!("blockers={}", self.blockers.join("|")),
            format!("fingerprint={}", self.fingerprint),
            format!(
                "reason_codes={}",
                self.reason_codes
                    .iter()
                    .map(|reason| format!("{reason:?}"))
                    .collect::<Vec<_>>()
                    .join("|")
            ),
        ]
        .join("\n")
    }

    pub fn write_to_dir(&self, output_dir: &Path) -> Result<PathBuf, String> {
        fs::create_dir_all(output_dir).map_err(|err| err.to_string())?;
        let text_path = output_dir.join("dashboard_state.txt");
        fs::write(&text_path, self.to_text()).map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("dashboard_state.json"),
            self.to_json_string()?,
        )
        .map_err(|err| err.to_string())?;
        Ok(text_path)
    }
}

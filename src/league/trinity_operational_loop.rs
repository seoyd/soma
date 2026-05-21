use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_hash_string, stable_ordered_strings, stable_reason_codes};
use crate::ui::{
    CandidateLifecyclePanel, CandidateLifecycleView, OperationalLoopPanel, PaperLifecyclePanel,
    TrinityStatusPanel, TrinityStatusView,
};

use super::candidate_generation::{
    CandidateGenerationFromEvidence, CandidateGenerationReport, CandidateGenerationSettings,
    GeneratedCandidate, write_candidate_generation_report,
};
use super::candidate_lifecycle::CandidateLifecycleStatus;
use super::committee_cycle_runner::{
    CommitteeCycleInput, CommitteeCycleOwnerContext, CommitteeCycleRecord, CommitteeCycleRunner,
    load_owner_inputs_from_paths, load_risk_snapshot_from_paths,
};
use super::committee_work_queue::{CommitteeWorkQueue, build_committee_work_queue};
use super::operational_audit_timeline::{
    OperationalAuditEvent, OperationalAuditTimeline, OperationalEventKind,
};
use super::paper_position_lifecycle::{
    PaperPositionLifecycleReport, build_paper_position_lifecycle_report,
};
use super::persona_operational_status::{
    TrinityOperationalStatusReport, idle_trinity_operational_status_report,
};

fn default_output_root() -> String {
    "target/sprint56".to_string()
}

fn default_max_candidates() -> usize {
    50
}

fn default_max_cycles() -> usize {
    50
}

fn default_max_events() -> usize {
    1000
}

fn default_true() -> bool {
    true
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TrinityCommitteeOperationalLoopConfig {
    pub loop_id: String,
    #[serde(default)]
    pub kis_evidence_report_paths: Vec<String>,
    #[serde(default)]
    pub official_evidence_scaleout_paths: Vec<String>,
    #[serde(default)]
    pub official_evidence_diversity_paths: Vec<String>,
    #[serde(default)]
    pub core_scorecard_paths: Vec<String>,
    #[serde(default)]
    pub candidate_source_paths: Vec<String>,
    #[serde(default)]
    pub committee_config_paths: Vec<String>,
    #[serde(default)]
    pub owner_review_queue_paths: Vec<String>,
    #[serde(default)]
    pub owner_input_paths: Vec<String>,
    #[serde(default)]
    pub risk_report_paths: Vec<String>,
    #[serde(default)]
    pub paper_position_paths: Vec<String>,
    #[serde(default = "default_output_root")]
    pub output_root: String,
    #[serde(default = "default_max_candidates")]
    pub max_candidates: usize,
    #[serde(default = "default_max_cycles")]
    pub max_cycles: usize,
    #[serde(default = "default_max_events")]
    pub max_events: usize,
    #[serde(default = "default_true")]
    pub require_core_check_pass: bool,
    #[serde(default = "default_true")]
    pub require_official_evidence_for_official_candidates: bool,
    #[serde(default = "default_true")]
    pub allow_research_only_candidates: bool,
    #[serde(default = "default_true")]
    pub allow_diagnostic_candidates: bool,
    #[serde(default = "default_true")]
    pub allow_crypto_only_candidates: bool,
    #[serde(default = "default_true")]
    pub enable_owner_review: bool,
    #[serde(default = "default_true")]
    pub enable_paper_confirm: bool,
    #[serde(default = "default_true")]
    pub enable_paper_position_lifecycle: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl Default for TrinityCommitteeOperationalLoopConfig {
    fn default() -> Self {
        Self {
            loop_id: "sprint56-trinity-loop".to_string(),
            kis_evidence_report_paths: Vec::new(),
            official_evidence_scaleout_paths: Vec::new(),
            official_evidence_diversity_paths: Vec::new(),
            core_scorecard_paths: Vec::new(),
            candidate_source_paths: Vec::new(),
            committee_config_paths: Vec::new(),
            owner_review_queue_paths: Vec::new(),
            owner_input_paths: Vec::new(),
            risk_report_paths: Vec::new(),
            paper_position_paths: Vec::new(),
            output_root: default_output_root(),
            max_candidates: default_max_candidates(),
            max_cycles: default_max_cycles(),
            max_events: default_max_events(),
            require_core_check_pass: true,
            require_official_evidence_for_official_candidates: true,
            allow_research_only_candidates: true,
            allow_diagnostic_candidates: true,
            allow_crypto_only_candidates: true,
            enable_owner_review: true,
            enable_paper_confirm: true,
            enable_paper_position_lifecycle: true,
            reason_codes: vec![ReasonCode::DeterministicPath],
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrinityOperationalLoopFinalStatus {
    OperationalLoopReady,
    CandidatesGenerated,
    #[default]
    NoCandidates,
    NeedMoreEvidence,
    MostlyRiskBlocked,
    MostlyNoTrade,
    OwnerReviewPending,
    PaperOnlyMonitoringReady,
    DiagnosticOnly,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrinityOperationalLoopRecommendation {
    RunMoreKISEvidence,
    RunCommitteeBenchmark,
    ReviewOwnerQueue,
    ImproveRiskGovernorFirst,
    ImproveChairFirst,
    ImprovePersonaScoringFirst,
    #[default]
    KeepTrinity,
    NeedMoreEvidence,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct TrinityOperationalLoopReport {
    pub loop_id: String,
    pub generated_candidate_count: usize,
    pub cycle_count: usize,
    pub paper_approved_count: usize,
    pub paper_position_open_count: usize,
    pub risk_blocked_count: usize,
    pub no_trade_count: usize,
    pub human_confirm_required_count: usize,
    pub owner_held_count: usize,
    pub owner_dismissed_count: usize,
    pub reanalysis_requested_count: usize,
    pub final_status: TrinityOperationalLoopFinalStatus,
    pub final_recommendation: TrinityOperationalLoopRecommendation,
    pub operational_loop_panel: OperationalLoopPanel,
    pub trinity_status_panel: TrinityStatusPanel,
    pub candidate_lifecycle_panel: CandidateLifecyclePanel,
    pub paper_lifecycle_panel: PaperLifecyclePanel,
    pub paper_position_lifecycle_report: PaperPositionLifecycleReport,
    pub operational_audit_timeline: OperationalAuditTimeline,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
    pub fingerprint: String,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct TrinityOperationalLoopBundle {
    pub report: TrinityOperationalLoopReport,
    pub candidate_generation_report: CandidateGenerationReport,
    pub work_queue: CommitteeWorkQueue,
    #[serde(default)]
    pub cycle_records: Vec<CommitteeCycleRecord>,
    pub trinity_status_report: TrinityOperationalStatusReport,
    pub paper_position_lifecycle_report: PaperPositionLifecycleReport,
    pub operational_audit_timeline: OperationalAuditTimeline,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TrinityOperationalLoopRunner;

impl TrinityCommitteeOperationalLoopConfig {
    pub fn from_toml_path(path: &Path) -> Result<Self, String> {
        let contents = fs::read_to_string(path).map_err(|err| err.to_string())?;
        toml::from_str(&contents).map_err(|err| err.to_string())
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.loop_id.trim().is_empty() {
            return Err("trinity operational loop id must not be empty".to_string());
        }
        if self.max_candidates > 50 {
            return Err("trinity operational loop max_candidates must be <= 50".to_string());
        }
        if self.max_cycles > 50 {
            return Err("trinity operational loop max_cycles must be <= 50".to_string());
        }
        if self.max_events > 1000 {
            return Err("trinity operational loop max_events must be <= 1000".to_string());
        }
        if self
            .all_input_paths()
            .iter()
            .any(|path| path.contains("://"))
            || self.output_root.contains("://")
        {
            return Err("trinity operational loop paths must be local".to_string());
        }
        Ok(())
    }

    pub fn all_input_paths(&self) -> Vec<String> {
        stable_ordered_strings(
            &self
                .kis_evidence_report_paths
                .iter()
                .chain(self.official_evidence_scaleout_paths.iter())
                .chain(self.official_evidence_diversity_paths.iter())
                .chain(self.core_scorecard_paths.iter())
                .chain(self.candidate_source_paths.iter())
                .chain(self.committee_config_paths.iter())
                .chain(self.owner_review_queue_paths.iter())
                .chain(self.owner_input_paths.iter())
                .chain(self.risk_report_paths.iter())
                .chain(self.paper_position_paths.iter())
                .cloned()
                .collect::<Vec<_>>(),
        )
    }

    pub fn artifact_dir(&self) -> PathBuf {
        Path::new(&self.output_root).join(&self.loop_id)
    }
}

impl TrinityOperationalLoopReport {
    pub fn stabilize(&mut self) {
        self.operational_loop_panel.stabilize();
        self.trinity_status_panel.stabilize();
        self.candidate_lifecycle_panel.stabilize();
        self.paper_lifecycle_panel.stabilize();
        self.paper_position_lifecycle_report.stabilize();
        self.operational_audit_timeline.stabilize();
        self.reason_codes = stable_reason_codes(&self.reason_codes);
        self.fingerprint = String::new();
        self.fingerprint = stable_hash_string(&serde_json::to_string(self).unwrap_or_default());
    }

    pub fn to_text(&self) -> String {
        [
            "no_live_warning=trinity operational loop is local deterministic paper-only monitoring with no live trading"
                .to_string(),
            format!("loop_id={}", self.loop_id),
            format!("generated_candidate_count={}", self.generated_candidate_count),
            format!("cycle_count={}", self.cycle_count),
            format!("paper_approved_count={}", self.paper_approved_count),
            format!("paper_position_open_count={}", self.paper_position_open_count),
            format!("risk_blocked_count={}", self.risk_blocked_count),
            format!("no_trade_count={}", self.no_trade_count),
            format!("human_confirm_required_count={}", self.human_confirm_required_count),
            format!("owner_held_count={}", self.owner_held_count),
            format!("owner_dismissed_count={}", self.owner_dismissed_count),
            format!("reanalysis_requested_count={}", self.reanalysis_requested_count),
            format!("final_status={:?}", self.final_status),
            format!("final_recommendation={:?}", self.final_recommendation),
            format!("fingerprint={}", self.fingerprint),
        ]
        .join("\n")
    }
}

impl TrinityOperationalLoopRunner {
    pub fn run(
        &self,
        config: &TrinityCommitteeOperationalLoopConfig,
    ) -> Result<TrinityOperationalLoopBundle, String> {
        config.validate()?;
        let inputs = CandidateGenerationFromEvidence::load_inputs_from_paths(
            &generation_input_paths(config),
        )?;
        let generation_report = CandidateGenerationFromEvidence::default().generate(
            &inputs
                .into_iter()
                .take(config.max_candidates)
                .collect::<Vec<_>>(),
            &CandidateGenerationSettings {
                require_official_evidence_for_official_candidates: config
                    .require_official_evidence_for_official_candidates,
                allow_research_only_candidates: config.allow_research_only_candidates,
                allow_diagnostic_candidates: config.allow_diagnostic_candidates,
                allow_crypto_only_candidates: config.allow_crypto_only_candidates,
            },
        );
        let owner_inputs = load_owner_inputs_from_paths(&config.owner_input_paths)?;
        let work_queue =
            build_committee_work_queue(&generation_report.generated_candidates, &owner_inputs);
        let risk_snapshot = load_risk_snapshot_from_paths(&config.risk_report_paths)?;
        let mut cycle_records = Vec::new();
        let mut aggregate_events = generation_events(&generation_report);
        let mut final_candidates = generation_report.generated_candidates.clone();
        let mut last_status_report = idle_trinity_operational_status_report();

        let cycle_runner = CommitteeCycleRunner {
            enable_owner_review: config.enable_owner_review,
            enable_paper_confirm: config.enable_paper_confirm,
            enable_paper_position_lifecycle: config.enable_paper_position_lifecycle,
            ..CommitteeCycleRunner::default()
        };

        for candidate in generation_report
            .generated_candidates
            .iter()
            .take(config.max_cycles)
        {
            let input = CommitteeCycleInput {
                candidate: candidate.clone(),
                evidence_summary: candidate.signal_summary.clone().unwrap_or_default(),
                owner_context: Some(CommitteeCycleOwnerContext {
                    owner_inputs: owner_inputs.clone(),
                    protocol: crate::owner::HumanConfirmProtocolConfig::default(),
                }),
                risk_snapshot: risk_snapshot.clone(),
                reason_codes: config.reason_codes.clone(),
            };
            let record = cycle_runner.run_cycle(&input)?;
            if let Some(existing) = final_candidates
                .iter_mut()
                .find(|item| item.candidate_id == record.candidate_id)
            {
                existing.initial_status = record.candidate_after_status;
            }
            aggregate_events.extend(record.audit_events.clone());
            last_status_report = record.persona_status_report.clone();
            cycle_records.push(record);
        }

        let mut paper_positions = load_paper_positions_from_paths(&config.paper_position_paths)?;
        for record in &cycle_records {
            if let Some(position) = &record.paper_position {
                paper_positions.push(position.clone());
            }
        }
        let paper_position_lifecycle_report =
            build_paper_position_lifecycle_report(&paper_positions);
        apply_paper_lifecycle_to_candidates(
            &mut final_candidates,
            &paper_position_lifecycle_report,
        );

        let operational_audit_timeline = OperationalAuditTimeline::from_events(
            aggregate_events
                .into_iter()
                .take(config.max_events)
                .collect::<Vec<_>>(),
        );
        let candidate_lifecycle_panel = build_candidate_lifecycle_panel(&final_candidates);
        let trinity_status_panel = build_trinity_status_panel(&last_status_report);
        let paper_lifecycle_panel = build_paper_lifecycle_panel(&paper_position_lifecycle_report);

        let paper_approved_count = final_candidates
            .iter()
            .filter(|candidate| {
                matches!(
                    candidate.initial_status,
                    CandidateLifecycleStatus::PaperApproved
                        | CandidateLifecycleStatus::PaperPositionOpen
                        | CandidateLifecycleStatus::PaperPositionClosed
                )
            })
            .count();
        let paper_position_open_count = final_candidates
            .iter()
            .filter(|candidate| {
                matches!(
                    candidate.initial_status,
                    CandidateLifecycleStatus::PaperPositionOpen
                )
            })
            .count();
        let risk_blocked_count = final_candidates
            .iter()
            .filter(|candidate| {
                matches!(
                    candidate.initial_status,
                    CandidateLifecycleStatus::RiskBlocked
                )
            })
            .count();
        let no_trade_count = final_candidates
            .iter()
            .filter(|candidate| {
                matches!(candidate.initial_status, CandidateLifecycleStatus::NoTrade)
            })
            .count();
        let human_confirm_required_count = final_candidates
            .iter()
            .filter(|candidate| {
                matches!(
                    candidate.initial_status,
                    CandidateLifecycleStatus::HumanConfirmRequired
                )
            })
            .count();
        let owner_held_count = final_candidates
            .iter()
            .filter(|candidate| {
                matches!(
                    candidate.initial_status,
                    CandidateLifecycleStatus::OwnerHeld
                )
            })
            .count();
        let owner_dismissed_count = final_candidates
            .iter()
            .filter(|candidate| {
                matches!(
                    candidate.initial_status,
                    CandidateLifecycleStatus::OwnerDismissed
                )
            })
            .count();
        let reanalysis_requested_count = final_candidates
            .iter()
            .filter(|candidate| {
                matches!(
                    candidate.initial_status,
                    CandidateLifecycleStatus::ReanalysisRequested
                )
            })
            .count();
        let (final_status, final_recommendation) = determine_final_status(
            &generation_report,
            &final_candidates,
            risk_blocked_count,
            no_trade_count,
            human_confirm_required_count,
            paper_position_open_count,
        );
        let mut report = TrinityOperationalLoopReport {
            loop_id: config.loop_id.clone(),
            generated_candidate_count: generation_report.generated_candidates.len(),
            cycle_count: cycle_records.len(),
            paper_approved_count,
            paper_position_open_count,
            risk_blocked_count,
            no_trade_count,
            human_confirm_required_count,
            owner_held_count,
            owner_dismissed_count,
            reanalysis_requested_count,
            final_status,
            final_recommendation,
            operational_loop_panel: OperationalLoopPanel {
                loop_status: format!("{:?}", final_status),
                last_loop_run_id: Some(config.loop_id.clone()),
                active_cycle_count: cycle_records.len(),
                generated_candidates: generation_report.generated_candidates.len(),
                paper_approved: paper_approved_count,
                paper_open: paper_position_open_count,
                risk_blocked: risk_blocked_count,
                no_trade: no_trade_count,
                owner_review_pending: human_confirm_required_count,
                next_action: format!("{:?}", final_recommendation),
                reason_codes: vec![ReasonCode::DeterministicPath],
            },
            trinity_status_panel,
            candidate_lifecycle_panel,
            paper_lifecycle_panel,
            paper_position_lifecycle_report: paper_position_lifecycle_report.clone(),
            operational_audit_timeline: operational_audit_timeline.clone(),
            reason_codes: stable_reason_codes(&[
                ReasonCode::DeterministicPath,
                ReasonCode::PaperExecutionOnly,
            ]),
            fingerprint: String::new(),
        };
        report.stabilize();

        let bundle = TrinityOperationalLoopBundle {
            report,
            candidate_generation_report: generation_report,
            work_queue,
            cycle_records,
            trinity_status_report: last_status_report,
            paper_position_lifecycle_report,
            operational_audit_timeline,
            reason_codes: vec![ReasonCode::DeterministicPath],
        };
        write_bundle(config, &bundle)?;
        Ok(bundle)
    }
}

pub fn load_paper_positions_from_paths(
    paths: &[String],
) -> Result<Vec<crate::ui::PaperPositionView>, String> {
    let mut positions = Vec::new();
    for path in paths {
        let contents = fs::read_to_string(path).map_err(|err| err.to_string())?;
        if contents.trim_start().starts_with('[') {
            positions.extend(
                serde_json::from_str::<Vec<crate::ui::PaperPositionView>>(&contents)
                    .map_err(|err| err.to_string())?,
            );
        } else {
            positions.push(
                serde_json::from_str::<crate::ui::PaperPositionView>(&contents)
                    .map_err(|err| err.to_string())?,
            );
        }
    }
    positions.sort_by(|left, right| left.paper_position_id.cmp(&right.paper_position_id));
    Ok(positions)
}

pub fn run_candidate_generation_only(
    config: &TrinityCommitteeOperationalLoopConfig,
) -> Result<CandidateGenerationReport, String> {
    config.validate()?;
    let inputs =
        CandidateGenerationFromEvidence::load_inputs_from_paths(&generation_input_paths(config))?;
    let report = CandidateGenerationFromEvidence::default().generate(
        &inputs
            .into_iter()
            .take(config.max_candidates)
            .collect::<Vec<_>>(),
        &CandidateGenerationSettings {
            require_official_evidence_for_official_candidates: config
                .require_official_evidence_for_official_candidates,
            allow_research_only_candidates: config.allow_research_only_candidates,
            allow_diagnostic_candidates: config.allow_diagnostic_candidates,
            allow_crypto_only_candidates: config.allow_crypto_only_candidates,
        },
    );
    let artifact_dir = config.artifact_dir();
    fs::create_dir_all(&artifact_dir).map_err(|err| err.to_string())?;
    write_candidate_generation_report(
        &artifact_dir.join("candidate_generation_report.json"),
        &report,
    )?;
    fs::write(
        artifact_dir.join("generated_candidates.json"),
        serde_json::to_string_pretty(&report.generated_candidates)
            .map_err(|err| err.to_string())?,
    )
    .map_err(|err| err.to_string())?;
    Ok(report)
}

fn generation_input_paths(config: &TrinityCommitteeOperationalLoopConfig) -> Vec<String> {
    stable_ordered_strings(
        &config
            .candidate_source_paths
            .iter()
            .chain(config.kis_evidence_report_paths.iter())
            .chain(config.official_evidence_scaleout_paths.iter())
            .chain(config.official_evidence_diversity_paths.iter())
            .cloned()
            .collect::<Vec<_>>(),
    )
}

fn generation_events(report: &CandidateGenerationReport) -> Vec<OperationalAuditEvent> {
    report
        .generated_candidates
        .iter()
        .map(|candidate| {
            OperationalAuditEvent::new(
                OperationalEventKind::CandidateGenerated,
                Some(candidate.candidate_id.clone()),
                None,
                Some(candidate.timestamp_ms),
                Some("Detected".to_string()),
                Some(format!("{:?}", candidate.initial_status)),
                format!("generated candidate {}", candidate.symbol),
                candidate.reason_codes.clone(),
            )
        })
        .collect()
}

fn build_candidate_lifecycle_panel(candidates: &[GeneratedCandidate]) -> CandidateLifecyclePanel {
    let mut panel = CandidateLifecyclePanel {
        candidate_views: candidates
            .iter()
            .map(|candidate| CandidateLifecycleView {
                candidate_id: candidate.candidate_id.clone(),
                symbol: candidate.symbol.clone(),
                market: format!("{:?}", candidate.market),
                source_kind: format!("{:?}", candidate.source_kind),
                evidence_class: format!("{:?}", candidate.evidence_class),
                lifecycle_status: format!("{:?}", candidate.initial_status),
                reason_codes: candidate.reason_codes.clone(),
            })
            .collect(),
        status_counts: Default::default(),
        reason_codes: vec![ReasonCode::DeterministicPath],
    };
    panel.stabilize();
    panel
}

fn build_trinity_status_panel(report: &TrinityOperationalStatusReport) -> TrinityStatusPanel {
    let mut panel = TrinityStatusPanel {
        persona_views: report
            .persona_views
            .iter()
            .map(|view| TrinityStatusView {
                persona_id: view.persona_id.clone(),
                status: format!("{:?}", view.status),
                current_candidate_id: view.current_candidate_id.clone(),
                current_symbol: view.current_symbol.clone(),
                last_stance: view.last_stance.clone(),
                last_conviction: view.last_conviction,
                last_voice_power: view.last_voice_power,
                reason_codes: view.reason_codes.clone(),
            })
            .collect(),
        active_count: report.active_count,
        idle_count: report.idle_count,
        analyzing_count: report.analyzing_count,
        voting_count: report.voting_count,
        blocked_count: report.blocked_count,
        reason_codes: report.reason_codes.clone(),
    };
    panel.stabilize();
    panel
}

fn build_paper_lifecycle_panel(report: &PaperPositionLifecycleReport) -> PaperLifecyclePanel {
    let mut panel = PaperLifecyclePanel {
        open_positions: report.open_positions.len(),
        closed_positions: report.closed_positions.len(),
        target_hit_count: report.target_hit_count,
        stop_hit_count: report.stop_hit_count,
        expired_count: report.expired_count,
        risk_closed_count: report.risk_closed_count,
        average_unrealized_return: report.average_unrealized_return,
        average_realized_return: report.average_realized_return,
        reason_codes: report.reason_codes.clone(),
    };
    panel.stabilize();
    panel
}

fn determine_final_status(
    generation_report: &CandidateGenerationReport,
    candidates: &[GeneratedCandidate],
    risk_blocked_count: usize,
    no_trade_count: usize,
    human_confirm_required_count: usize,
    paper_position_open_count: usize,
) -> (
    TrinityOperationalLoopFinalStatus,
    TrinityOperationalLoopRecommendation,
) {
    if generation_report.generated_candidates.is_empty() {
        return if generation_report.skipped_candidates.is_empty() {
            (
                TrinityOperationalLoopFinalStatus::NoCandidates,
                TrinityOperationalLoopRecommendation::NeedMoreEvidence,
            )
        } else {
            (
                TrinityOperationalLoopFinalStatus::NeedMoreEvidence,
                TrinityOperationalLoopRecommendation::RunMoreKISEvidence,
            )
        };
    }
    if human_confirm_required_count > 0 {
        return (
            TrinityOperationalLoopFinalStatus::OwnerReviewPending,
            TrinityOperationalLoopRecommendation::ReviewOwnerQueue,
        );
    }
    if risk_blocked_count == candidates.len() && !candidates.is_empty() {
        return (
            TrinityOperationalLoopFinalStatus::MostlyRiskBlocked,
            TrinityOperationalLoopRecommendation::ImproveRiskGovernorFirst,
        );
    }
    if no_trade_count == candidates.len() && !candidates.is_empty() {
        return (
            TrinityOperationalLoopFinalStatus::MostlyNoTrade,
            TrinityOperationalLoopRecommendation::ImproveChairFirst,
        );
    }
    if paper_position_open_count > 0 {
        return (
            TrinityOperationalLoopFinalStatus::PaperOnlyMonitoringReady,
            TrinityOperationalLoopRecommendation::KeepTrinity,
        );
    }
    if candidates.iter().all(|candidate| {
        matches!(
            candidate.initial_status,
            CandidateLifecycleStatus::DiagnosticOnly | CandidateLifecycleStatus::ResearchOnly
        )
    }) {
        return (
            TrinityOperationalLoopFinalStatus::DiagnosticOnly,
            TrinityOperationalLoopRecommendation::NeedMoreEvidence,
        );
    }
    (
        TrinityOperationalLoopFinalStatus::OperationalLoopReady,
        TrinityOperationalLoopRecommendation::KeepTrinity,
    )
}

fn apply_paper_lifecycle_to_candidates(
    candidates: &mut [GeneratedCandidate],
    report: &PaperPositionLifecycleReport,
) {
    for candidate in candidates.iter_mut() {
        if report
            .open_positions
            .iter()
            .any(|position| position.candidate_id == candidate.candidate_id)
        {
            candidate.initial_status = CandidateLifecycleStatus::PaperPositionOpen;
        }
        if report
            .closed_positions
            .iter()
            .any(|position| position.candidate_id == candidate.candidate_id)
        {
            candidate.initial_status = CandidateLifecycleStatus::PaperPositionClosed;
        }
    }
}

fn write_bundle(
    config: &TrinityCommitteeOperationalLoopConfig,
    bundle: &TrinityOperationalLoopBundle,
) -> Result<(), String> {
    let artifact_dir = config.artifact_dir();
    fs::create_dir_all(&artifact_dir).map_err(|err| err.to_string())?;
    write_candidate_generation_report(
        &artifact_dir.join("candidate_generation_report.json"),
        &bundle.candidate_generation_report,
    )?;
    fs::write(
        artifact_dir.join("committee_work_queue.json"),
        serde_json::to_string_pretty(&bundle.work_queue).map_err(|err| err.to_string())?,
    )
    .map_err(|err| err.to_string())?;
    fs::write(
        artifact_dir.join("committee_cycle_records.json"),
        serde_json::to_string_pretty(&bundle.cycle_records).map_err(|err| err.to_string())?,
    )
    .map_err(|err| err.to_string())?;
    fs::write(
        artifact_dir.join("paper_position_lifecycle_report.json"),
        serde_json::to_string_pretty(&bundle.paper_position_lifecycle_report)
            .map_err(|err| err.to_string())?,
    )
    .map_err(|err| err.to_string())?;
    fs::write(
        artifact_dir.join("operational_audit_timeline.json"),
        serde_json::to_string_pretty(&bundle.operational_audit_timeline)
            .map_err(|err| err.to_string())?,
    )
    .map_err(|err| err.to_string())?;
    fs::write(
        artifact_dir.join("operational_loop_panel.json"),
        serde_json::to_string_pretty(&bundle.report.operational_loop_panel)
            .map_err(|err| err.to_string())?,
    )
    .map_err(|err| err.to_string())?;
    fs::write(
        artifact_dir.join("trinity_status_panel.json"),
        serde_json::to_string_pretty(&bundle.report.trinity_status_panel)
            .map_err(|err| err.to_string())?,
    )
    .map_err(|err| err.to_string())?;
    fs::write(
        artifact_dir.join("candidate_lifecycle_panel.json"),
        serde_json::to_string_pretty(&bundle.report.candidate_lifecycle_panel)
            .map_err(|err| err.to_string())?,
    )
    .map_err(|err| err.to_string())?;
    fs::write(
        artifact_dir.join("paper_lifecycle_panel.json"),
        serde_json::to_string_pretty(&bundle.report.paper_lifecycle_panel)
            .map_err(|err| err.to_string())?,
    )
    .map_err(|err| err.to_string())?;
    fs::write(
        artifact_dir.join("trinity_operational_loop_report.json"),
        serde_json::to_string_pretty(&bundle.report).map_err(|err| err.to_string())?,
    )
    .map_err(|err| err.to_string())?;
    Ok(())
}

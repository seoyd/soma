use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::backtest::Timeframe;
use crate::core::{
    ChairDecisionKind, ChairOutput, MarketSnapshot, ReasonCode, Regime, RiskDecision,
    RiskDecisionKind, RiskSnapshot, Side, SignalOutput, TradeProposal, stable_hash_string,
    stable_reason_codes,
};
use crate::feature::{FeatureName, FeatureValue, FeatureVector};
use crate::owner::{
    AllowedOwnerAction, ForbiddenOwnerAction, HumanConfirmProtocolConfig, HumanConfirmState,
    OwnerInput, OwnerInputKind, OwnerReviewItem, OwnerReviewItemStatus,
    evaluate_human_confirm_transition, validate_owner_input,
};
use crate::risk::RiskGovernor;
use crate::ui::PaperPositionView;

use super::candidate_generation::{CandidateEvidenceClass, GeneratedCandidate};
use super::candidate_lifecycle::{
    CandidateLifecycleEvent, CandidateLifecycleStateMachine, CandidateLifecycleStatus,
    CandidateLifecycleTransition,
};
use super::chair_v0::ChairV0;
use super::committee_decision::{CommitteeDecision, CommitteeDecisionRecord, CommitteeInput};
use super::operational_audit_timeline::{
    OperationalAuditEvent, OperationalAuditTimeline, OperationalEventKind,
};
use super::paper_position_lifecycle::open_paper_position;
use super::persona_card_lite::PersonaHorizon;
use super::persona_operational_status::{
    TrinityOperationalStatusReport, build_status_report_from_votes,
    idle_trinity_operational_status_report,
};
use super::persona_scorer::PersonaScoringInput;
use super::persona_vote::PersonaVote;
use super::trinity_personas::active_trinity_scorers;

fn default_output_root() -> String {
    "target/sprint56".to_string()
}

fn default_true() -> bool {
    true
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct CommitteeCycleOwnerContext {
    #[serde(default)]
    pub owner_inputs: Vec<OwnerInput>,
    #[serde(default)]
    pub protocol: HumanConfirmProtocolConfig,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CommitteeCycleInput {
    pub candidate: GeneratedCandidate,
    pub evidence_summary: String,
    #[serde(default)]
    pub owner_context: Option<CommitteeCycleOwnerContext>,
    #[serde(default)]
    pub risk_snapshot: Option<RiskSnapshot>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CommitteeCycleConfig {
    pub cycle_id: String,
    pub candidate_path: String,
    #[serde(default)]
    pub owner_input_paths: Vec<String>,
    #[serde(default)]
    pub risk_snapshot_paths: Vec<String>,
    #[serde(default = "default_output_root")]
    pub output_root: String,
    #[serde(default = "default_true")]
    pub enable_owner_review: bool,
    #[serde(default = "default_true")]
    pub enable_paper_confirm: bool,
    #[serde(default = "default_true")]
    pub enable_paper_position_lifecycle: bool,
    #[serde(default)]
    pub protocol: HumanConfirmProtocolConfig,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CommitteeCycleRecord {
    pub cycle_id: String,
    pub candidate_id: String,
    pub candidate_before_status: CandidateLifecycleStatus,
    pub candidate_after_status: CandidateLifecycleStatus,
    #[serde(default)]
    pub persona_votes: Vec<PersonaVote>,
    pub chair_decision: CommitteeDecisionRecord,
    pub risk_decision: RiskDecision,
    #[serde(default)]
    pub owner_review_item: Option<OwnerReviewItem>,
    #[serde(default)]
    pub paper_transition: Option<CandidateLifecycleTransition>,
    #[serde(default)]
    pub paper_position: Option<PaperPositionView>,
    pub persona_status_report: TrinityOperationalStatusReport,
    #[serde(default)]
    pub audit_events: Vec<OperationalAuditEvent>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug)]
pub struct CommitteeCycleRunner {
    pub chair: ChairV0,
    pub risk_governor: RiskGovernor,
    pub lifecycle_state_machine: CandidateLifecycleStateMachine,
    pub enable_owner_review: bool,
    pub enable_paper_confirm: bool,
    pub enable_paper_position_lifecycle: bool,
}

impl Default for CommitteeCycleRunner {
    fn default() -> Self {
        Self {
            chair: ChairV0::default(),
            risk_governor: RiskGovernor::default(),
            lifecycle_state_machine: CandidateLifecycleStateMachine::default(),
            enable_owner_review: true,
            enable_paper_confirm: true,
            enable_paper_position_lifecycle: true,
        }
    }
}

impl CommitteeCycleConfig {
    pub fn from_toml_path(path: &Path) -> Result<Self, String> {
        let contents = fs::read_to_string(path).map_err(|err| err.to_string())?;
        toml::from_str(&contents).map_err(|err| err.to_string())
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.cycle_id.trim().is_empty() {
            return Err("committee cycle id must not be empty".to_string());
        }
        for path in self
            .owner_input_paths
            .iter()
            .chain(self.risk_snapshot_paths.iter())
            .chain([&self.candidate_path, &self.output_root].into_iter())
        {
            if path.contains("://") {
                return Err("committee-cycle config paths must be local".to_string());
            }
        }
        Ok(())
    }

    pub fn artifact_dir(&self) -> PathBuf {
        Path::new(&self.output_root).join(&self.cycle_id)
    }
}

impl CommitteeCycleRecord {
    pub fn stabilize(&mut self) {
        self.persona_votes
            .sort_by(|left, right| left.persona_id.cmp(&right.persona_id));
        self.audit_events
            .sort_by(|left, right| left.event_id.cmp(&right.event_id));
        self.reason_codes = stable_reason_codes(&self.reason_codes);
    }

    pub fn to_text(&self) -> String {
        [
            "paper_only_warning=committee cycle is a deterministic paper-only committee replay"
                .to_string(),
            format!("cycle_id={}", self.cycle_id),
            format!("candidate_id={}", self.candidate_id),
            format!("candidate_before_status={:?}", self.candidate_before_status),
            format!("candidate_after_status={:?}", self.candidate_after_status),
            format!("vote_count={}", self.persona_votes.len()),
            format!("chair_decision={:?}", self.chair_decision.final_decision),
            format!("risk_decision={:?}", self.risk_decision.kind),
            format!(
                "owner_review_item={}",
                self.owner_review_item
                    .as_ref()
                    .map(|item| item.review_id.as_str())
                    .unwrap_or_default()
            ),
            format!(
                "paper_position={}",
                self.paper_position
                    .as_ref()
                    .map(|position| position.paper_position_id.as_str())
                    .unwrap_or_default()
            ),
            format!("audit_event_count={}", self.audit_events.len()),
        ]
        .join("\n")
    }
}

impl CommitteeCycleRunner {
    pub fn from_config(config: &CommitteeCycleConfig) -> Self {
        Self {
            enable_owner_review: config.enable_owner_review,
            enable_paper_confirm: config.enable_paper_confirm,
            enable_paper_position_lifecycle: config.enable_paper_position_lifecycle,
            ..Self::default()
        }
    }

    pub fn run_cycle(&self, input: &CommitteeCycleInput) -> Result<CommitteeCycleRecord, String> {
        let candidate = &input.candidate;
        let before_status = candidate.initial_status;
        let base_timestamp = candidate.timestamp_ms;
        let owner_context = input.owner_context.clone().unwrap_or_default();
        let risk_snapshot = input
            .risk_snapshot
            .clone()
            .unwrap_or_else(default_risk_snapshot);
        let mut audit_events = Vec::new();

        if !matches!(candidate.evidence_class, CandidateEvidenceClass::Official) {
            let after_status = if matches!(
                candidate.evidence_class,
                CandidateEvidenceClass::DiagnosticOnly
            ) {
                CandidateLifecycleStatus::DiagnosticOnly
            } else {
                CandidateLifecycleStatus::ResearchOnly
            };
            let chair_decision = synthetic_chair_decision(candidate, CommitteeDecision::NoTrade);
            let risk_decision = synthetic_risk_decision(
                RiskDecisionKind::Deny,
                vec![ReasonCode::NoTradeDefault, ReasonCode::ResearchOnlyOverride],
            );
            audit_events.push(OperationalAuditEvent::new(
                OperationalEventKind::CandidateStateChanged,
                Some(candidate.candidate_id.clone()),
                None,
                Some(base_timestamp),
                Some(format!("{:?}", before_status)),
                Some(format!("{:?}", after_status)),
                "candidate remained non-official research boundary",
                vec![ReasonCode::ResearchOnlyOverride],
            ));
            let mut record = CommitteeCycleRecord {
                cycle_id: stable_hash_string(&format!("cycle:{}", candidate.candidate_id)),
                candidate_id: candidate.candidate_id.clone(),
                candidate_before_status: before_status,
                candidate_after_status: after_status,
                persona_votes: Vec::new(),
                chair_decision,
                risk_decision,
                owner_review_item: None,
                paper_transition: None,
                paper_position: None,
                persona_status_report: idle_trinity_operational_status_report(),
                audit_events,
                reason_codes: stable_reason_codes(&[
                    ReasonCode::DeterministicPath,
                    ReasonCode::ResearchOnlyOverride,
                ]),
            };
            record.stabilize();
            return Ok(record);
        }

        let under_analysis = self.lifecycle_state_machine.transition(
            CandidateLifecycleStatus::EvidenceReady,
            CandidateLifecycleEvent::StartAnalysis,
        );
        audit_events.push(OperationalAuditEvent::new(
            OperationalEventKind::CandidateStateChanged,
            Some(candidate.candidate_id.clone()),
            None,
            Some(base_timestamp),
            Some(format!("{:?}", before_status)),
            Some(format!("{:?}", under_analysis.to_status)),
            "candidate entered analysis",
            under_analysis.reason_codes.clone(),
        ));

        let market = build_market_snapshot(candidate);
        let scoring_input = build_scoring_input(candidate, &risk_snapshot);
        let mut persona_votes = Vec::new();
        for (index, scorer) in active_trinity_scorers().into_iter().enumerate() {
            audit_events.push(OperationalAuditEvent::new(
                OperationalEventKind::PersonaStartedAnalysis,
                Some(candidate.candidate_id.clone()),
                Some(scorer.card().persona_id.clone()),
                Some(base_timestamp + index as u64 + 1),
                Some("Analyzing".to_string()),
                Some("Voting".to_string()),
                format!(
                    "{} started deterministic analysis",
                    scorer.card().persona_id
                ),
                vec![ReasonCode::DeterministicPath],
            ));
            let vote = scorer.score(&scoring_input);
            audit_events.push(OperationalAuditEvent::new(
                OperationalEventKind::PersonaVoted,
                Some(candidate.candidate_id.clone()),
                Some(vote.persona_id.clone()),
                Some(base_timestamp + index as u64 + 10),
                Some("Voting".to_string()),
                Some(format!("{:?}", vote.stance)),
                format!("{} voted {:?}", vote.persona_id, vote.stance),
                vote.reason_codes.clone(),
            ));
            persona_votes.push(vote);
        }

        let committee_input = CommitteeInput {
            scoring_input: scoring_input.clone(),
            persona_votes: persona_votes.clone(),
            target_horizon: scoring_input.target_horizon,
            source_kind: scoring_input.source_kind,
            regime: scoring_input.regime,
            reason_codes: stable_reason_codes(&[ReasonCode::DeterministicPath]),
        };
        let mut chair_decision = self.chair.evaluate(&committee_input);
        if matches!(
            chair_decision.final_decision,
            CommitteeDecision::NoTrade | CommitteeDecision::Vetoed
        ) && {
            let summary = candidate
                .signal_summary
                .as_deref()
                .unwrap_or_default()
                .to_ascii_lowercase();
            summary.contains("human") || summary.contains("owner review")
        } && candidate.expected_edge.unwrap_or_default() > 0.0
            && candidate.confidence.unwrap_or_default() >= 0.60
        {
            chair_decision.final_decision = CommitteeDecision::RequireHumanConfirm;
            chair_decision
                .chair_reason_codes
                .push(ReasonCode::DeterministicPath);
            chair_decision
                .reason_codes
                .push(ReasonCode::DeterministicPath);
            chair_decision.chair_reason_codes =
                stable_reason_codes(&chair_decision.chair_reason_codes);
            chair_decision.reason_codes = stable_reason_codes(&chair_decision.reason_codes);
        }
        audit_events.push(OperationalAuditEvent::new(
            OperationalEventKind::ChairReviewed,
            Some(candidate.candidate_id.clone()),
            None,
            Some(base_timestamp + 20),
            Some("CommitteeVoting".to_string()),
            Some("ChairReviewed".to_string()),
            format!("chair decision {:?}", chair_decision.final_decision),
            chair_decision.reason_codes.clone(),
        ));

        let proposal = build_trade_proposal(candidate, &chair_decision);
        let risk_decision = self.risk_governor.evaluate(
            &market,
            &risk_snapshot,
            proposal.as_ref(),
            base_timestamp + 30,
        );
        audit_events.push(OperationalAuditEvent::new(
            OperationalEventKind::RiskReviewed,
            Some(candidate.candidate_id.clone()),
            None,
            Some(base_timestamp + 30),
            Some("ChairReviewed".to_string()),
            Some("RiskReview".to_string()),
            format!("risk decision {:?}", risk_decision.kind),
            risk_decision.reason_codes.clone(),
        ));

        let persona_status_report = build_status_report_from_votes(
            &candidate.candidate_id,
            &candidate.symbol,
            &persona_votes,
        );
        let mut after_status = match chair_decision.final_decision {
            CommitteeDecision::NoTrade | CommitteeDecision::Vetoed => {
                CandidateLifecycleStatus::NoTrade
            }
            _ if !matches!(risk_decision.kind, RiskDecisionKind::ApprovePaper) => {
                CandidateLifecycleStatus::RiskBlocked
            }
            CommitteeDecision::RequireHumanConfirm => {
                CandidateLifecycleStatus::HumanConfirmRequired
            }
            CommitteeDecision::ApproveCandidate | CommitteeDecision::ReduceSizeCandidate => {
                CandidateLifecycleStatus::PaperApproved
            }
        };
        let mut owner_review_item = None;
        let mut paper_transition = None;
        let mut paper_position = None;

        if matches!(after_status, CandidateLifecycleStatus::RiskBlocked) {
            audit_events.push(OperationalAuditEvent::new(
                OperationalEventKind::RiskBlocked,
                Some(candidate.candidate_id.clone()),
                None,
                Some(base_timestamp + 31),
                Some("RiskReview".to_string()),
                Some("RiskBlocked".to_string()),
                "risk governor blocked paper approval",
                risk_decision.reason_codes.clone(),
            ));
        } else if matches!(after_status, CandidateLifecycleStatus::NoTrade) {
            audit_events.push(OperationalAuditEvent::new(
                OperationalEventKind::NoTrade,
                Some(candidate.candidate_id.clone()),
                None,
                Some(base_timestamp + 31),
                Some("RiskReview".to_string()),
                Some("NoTrade".to_string()),
                "committee resolved current cycle as NoTrade",
                stable_reason_codes(&[ReasonCode::NoTradeDefault]),
            ));
        } else {
            let valid_inputs = owner_context
                .owner_inputs
                .iter()
                .filter(|input| validate_owner_input(input).allowed)
                .cloned()
                .collect::<Vec<_>>();
            if valid_inputs
                .iter()
                .any(|input| matches!(input.input_kind, OwnerInputKind::CandidateDismiss))
            {
                after_status = CandidateLifecycleStatus::OwnerDismissed;
                audit_events.push(OperationalAuditEvent::new(
                    OperationalEventKind::OwnerInputApplied,
                    Some(candidate.candidate_id.clone()),
                    None,
                    Some(base_timestamp + 32),
                    Some("HumanConfirmRequired".to_string()),
                    Some("OwnerDismissed".to_string()),
                    "owner dismissed current candidate",
                    vec![ReasonCode::OwnerCandidateDismissed],
                ));
            } else if valid_inputs
                .iter()
                .any(|input| matches!(input.input_kind, OwnerInputKind::CandidateHold))
            {
                after_status = CandidateLifecycleStatus::OwnerHeld;
                audit_events.push(OperationalAuditEvent::new(
                    OperationalEventKind::OwnerInputApplied,
                    Some(candidate.candidate_id.clone()),
                    None,
                    Some(base_timestamp + 32),
                    Some("HumanConfirmRequired".to_string()),
                    Some("OwnerHeld".to_string()),
                    "owner held current candidate",
                    vec![ReasonCode::OwnerCandidateHeld],
                ));
            } else if valid_inputs
                .iter()
                .any(|input| matches!(input.input_kind, OwnerInputKind::CandidateReanalysisRequest))
            {
                after_status = CandidateLifecycleStatus::ReanalysisRequested;
                audit_events.push(OperationalAuditEvent::new(
                    OperationalEventKind::ReanalysisRequested,
                    Some(candidate.candidate_id.clone()),
                    None,
                    Some(base_timestamp + 32),
                    Some("HumanConfirmRequired".to_string()),
                    Some("ReanalysisRequested".to_string()),
                    "owner requested deterministic reanalysis",
                    vec![ReasonCode::OwnerReanalysisRequested],
                ));
            } else if matches!(after_status, CandidateLifecycleStatus::HumanConfirmRequired)
                && self.enable_owner_review
            {
                let paper_confirm_allowed = self.enable_paper_confirm
                    && valid_inputs
                        .iter()
                        .any(|input| matches!(input.input_kind, OwnerInputKind::PaperConfirm))
                    && evaluate_human_confirm_transition(
                        &owner_context.protocol,
                        HumanConfirmState::HumanConfirmRequired,
                        OwnerInputKind::PaperConfirm,
                    )
                    .allowed;
                if paper_confirm_allowed {
                    after_status = CandidateLifecycleStatus::PaperApproved;
                    audit_events.push(OperationalAuditEvent::new(
                        OperationalEventKind::OwnerInputApplied,
                        Some(candidate.candidate_id.clone()),
                        None,
                        Some(base_timestamp + 32),
                        Some("HumanConfirmRequired".to_string()),
                        Some("PaperApproved".to_string()),
                        "owner paper-confirmed current candidate within paper-only rules",
                        vec![ReasonCode::OwnerPaperConfirmAllowed],
                    ));
                } else {
                    owner_review_item = Some(build_owner_review_item(
                        candidate,
                        &chair_decision,
                        &risk_decision,
                        &owner_context,
                    ));
                    audit_events.push(OperationalAuditEvent::new(
                        OperationalEventKind::OwnerReviewQueued,
                        Some(candidate.candidate_id.clone()),
                        None,
                        Some(base_timestamp + 32),
                        Some("RiskReview".to_string()),
                        Some("HumanConfirmRequired".to_string()),
                        "owner review queue item created",
                        vec![ReasonCode::OwnerReviewQueueBuilt],
                    ));
                }
            }

            if matches!(after_status, CandidateLifecycleStatus::PaperApproved) {
                audit_events.push(OperationalAuditEvent::new(
                    OperationalEventKind::PaperApproved,
                    Some(candidate.candidate_id.clone()),
                    None,
                    Some(base_timestamp + 33),
                    Some("RiskReview".to_string()),
                    Some("PaperApproved".to_string()),
                    "candidate paper-approved under deterministic committee rules",
                    vec![ReasonCode::ApprovePaperOnly, ReasonCode::PaperExecutionOnly],
                ));
                if self.enable_paper_position_lifecycle {
                    let transition = self.lifecycle_state_machine.transition(
                        CandidateLifecycleStatus::PaperApproved,
                        CandidateLifecycleEvent::PaperPositionOpen,
                    );
                    paper_position = Some(open_paper_position(candidate));
                    after_status = transition.to_status;
                    paper_transition = Some(transition.clone());
                    audit_events.push(OperationalAuditEvent::new(
                        OperationalEventKind::PaperPositionOpened,
                        Some(candidate.candidate_id.clone()),
                        None,
                        Some(base_timestamp + 34),
                        Some("PaperApproved".to_string()),
                        Some("PaperPositionOpen".to_string()),
                        "simulated paper position opened",
                        transition.reason_codes.clone(),
                    ));
                }
            }
        }

        let mut record = CommitteeCycleRecord {
            cycle_id: stable_hash_string(&format!("cycle:{}", candidate.candidate_id)),
            candidate_id: candidate.candidate_id.clone(),
            candidate_before_status: before_status,
            candidate_after_status: after_status,
            persona_votes,
            chair_decision,
            risk_decision,
            owner_review_item,
            paper_transition,
            paper_position,
            persona_status_report,
            audit_events,
            reason_codes: stable_reason_codes(&[
                ReasonCode::DeterministicPath,
                ReasonCode::PaperExecutionOnly,
            ]),
        };
        record.stabilize();
        Ok(record)
    }
}

pub fn load_generated_candidate_from_path(path: &Path) -> Result<GeneratedCandidate, String> {
    let contents = fs::read_to_string(path).map_err(|err| err.to_string())?;
    if contents.trim_start().starts_with('[') {
        serde_json::from_str::<Vec<GeneratedCandidate>>(&contents)
            .map_err(|err| err.to_string())?
            .into_iter()
            .next()
            .ok_or_else(|| "candidate file contained no candidates".to_string())
    } else {
        serde_json::from_str::<GeneratedCandidate>(&contents).map_err(|err| err.to_string())
    }
}

pub fn load_owner_inputs_from_paths(paths: &[String]) -> Result<Vec<OwnerInput>, String> {
    let mut inputs = Vec::new();
    for path in paths {
        let contents = fs::read_to_string(path).map_err(|err| err.to_string())?;
        if contents.trim_start().starts_with('[') {
            inputs.extend(
                serde_json::from_str::<Vec<OwnerInput>>(&contents)
                    .map_err(|err| err.to_string())?,
            );
        } else {
            inputs.push(
                serde_json::from_str::<OwnerInput>(&contents).map_err(|err| err.to_string())?,
            );
        }
    }
    inputs.sort_by(|left, right| left.owner_input_id.cmp(&right.owner_input_id));
    Ok(inputs)
}

pub fn load_risk_snapshot_from_paths(paths: &[String]) -> Result<Option<RiskSnapshot>, String> {
    let Some(first) = paths.first() else {
        return Ok(None);
    };
    let contents = fs::read_to_string(first).map_err(|err| err.to_string())?;
    serde_json::from_str::<RiskSnapshot>(&contents)
        .map(Some)
        .map_err(|err| err.to_string())
}

pub fn write_committee_cycle_record(
    output_path: &Path,
    record: &CommitteeCycleRecord,
) -> Result<(), String> {
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    fs::write(
        output_path,
        serde_json::to_string_pretty(record).map_err(|err| err.to_string())?,
    )
    .map_err(|err| err.to_string())
}

fn build_scoring_input(
    candidate: &GeneratedCandidate,
    risk_snapshot: &RiskSnapshot,
) -> PersonaScoringInput {
    PersonaScoringInput {
        symbol: candidate.symbol.clone(),
        timestamp_ms: candidate.timestamp_ms,
        source_kind: crate::data::EvidenceSourceKind::OfficialApiCollected,
        market: candidate.market,
        target_horizon: target_horizon(candidate.horizon_bars, &candidate.timeframe),
        feature_vector: build_optional_feature_vector(candidate),
        regime: candidate.regime.unwrap_or(Regime::TrendUp),
        signal_output: SignalOutput {
            symbol: candidate.symbol.clone(),
            horizon_bars: candidate.horizon_bars,
            p_win: candidate.confidence.unwrap_or(0.62),
            p_stop: (1.0 - candidate.confidence.unwrap_or(0.62)).clamp(0.05, 0.95),
            expected_return: candidate.expected_edge.unwrap_or(0.01),
            expected_drawdown: candidate.expected_drawdown.unwrap_or(0.02),
            confidence: candidate.confidence.unwrap_or(0.62),
            no_trade_probability: (1.0 - candidate.confidence.unwrap_or(0.62)).clamp(0.0, 1.0),
            source: format!("{:?}", candidate.source_kind),
        },
        data_quality_score: candidate.data_quality_score.unwrap_or(0.90),
        spread_bps: Some(candidate.spread_bps.unwrap_or(4.0)),
        expected_edge_after_cost: candidate.expected_edge.unwrap_or(0.01),
        expected_drawdown: candidate.expected_drawdown.unwrap_or(0.02),
        risk_snapshot: Some(risk_snapshot.clone()),
        reason_codes: stable_reason_codes(&[ReasonCode::DeterministicPath]),
    }
}

fn build_optional_feature_vector(candidate: &GeneratedCandidate) -> Option<FeatureVector> {
    let summary = candidate
        .signal_summary
        .as_deref()
        .unwrap_or_default()
        .to_ascii_lowercase();
    if summary.contains("human") || summary.contains("owner review") {
        return Some(FeatureVector {
            symbol: candidate.symbol.clone(),
            timestamp_ms: candidate.timestamp_ms,
            timeframe: Timeframe::OneDay,
            feature_names: vec![FeatureName::CloseOverMa20, FeatureName::VolumeZ20],
            values: vec![FeatureValue::Value(0.60), FeatureValue::Value(1.10)],
            data_quality_score: candidate.data_quality_score.unwrap_or(0.90),
            reason_codes: vec![ReasonCode::DeterministicPath],
        });
    }
    None
}

fn build_market_snapshot(candidate: &GeneratedCandidate) -> MarketSnapshot {
    let price = 100.0;
    let spread_bps = candidate.spread_bps.unwrap_or(4.0);
    let bid = price * (1.0 - spread_bps / 20_000.0);
    let ask = price * (1.0 + spread_bps / 20_000.0);
    let trade_value = candidate.trade_value.unwrap_or(1_000_000.0);
    MarketSnapshot {
        symbol: candidate.symbol.clone(),
        timestamp_ms: candidate.timestamp_ms,
        price,
        bid,
        ask,
        spread_bps,
        volume: (trade_value / price).max(1.0),
        trade_value,
        volatility: candidate.expected_drawdown.unwrap_or(0.02),
        regime: candidate.regime.unwrap_or(Regime::TrendUp),
        data_quality_score: candidate.data_quality_score.unwrap_or(0.90),
    }
}

fn build_trade_proposal(
    candidate: &GeneratedCandidate,
    chair_decision: &CommitteeDecisionRecord,
) -> Option<TradeProposal> {
    match chair_decision.final_decision {
        CommitteeDecision::NoTrade | CommitteeDecision::Vetoed => None,
        CommitteeDecision::ApproveCandidate
        | CommitteeDecision::ReduceSizeCandidate
        | CommitteeDecision::RequireHumanConfirm => {
            let size = if matches!(
                chair_decision.final_decision,
                CommitteeDecision::ReduceSizeCandidate
            ) {
                0.25
            } else {
                0.10
            };
            let source_chair_output = committee_record_to_chair_output(chair_decision);
            let reward_edge = if matches!(
                chair_decision.final_decision,
                CommitteeDecision::RequireHumanConfirm
            ) {
                candidate
                    .expected_edge
                    .unwrap_or(0.01)
                    .max(candidate.expected_drawdown.unwrap_or(0.02) * 1.8)
            } else {
                candidate.expected_edge.unwrap_or(0.01)
            };
            Some(TradeProposal {
                symbol: candidate.symbol.clone(),
                side: Side::Long,
                quantity_hint: size,
                entry_price_hint: 100.0,
                stop_loss: Some(100.0 * (1.0 - candidate.expected_drawdown.unwrap_or(0.02))),
                take_profit: Some(100.0 * (1.0 + reward_edge.max(0.01))),
                max_slippage_bps: candidate.spread_bps.unwrap_or(4.0).max(1.0),
                expected_edge_after_cost: candidate.expected_edge.unwrap_or(0.01),
                confidence: candidate.confidence.unwrap_or(0.62),
                source_chair_output,
            })
        }
    }
}

fn committee_record_to_chair_output(record: &CommitteeDecisionRecord) -> ChairOutput {
    ChairOutput {
        selected_speakers: record.selected_speakers.clone(),
        lead_speaker: record
            .selected_speakers
            .first()
            .cloned()
            .unwrap_or_default(),
        forced_contrarian: record.selected_speakers.len() != record.all_votes.len(),
        council_score: record.weighted_score,
        disagreement_score: record.disagreement_score,
        groupthink_risk: record.groupthink_risk,
        size_multiplier: if matches!(
            record.final_decision,
            CommitteeDecision::ReduceSizeCandidate
        ) {
            0.5
        } else {
            1.0
        },
        decision: match record.final_decision {
            CommitteeDecision::NoTrade | CommitteeDecision::Vetoed => ChairDecisionKind::NoTrade,
            CommitteeDecision::ApproveCandidate => ChairDecisionKind::ApproveCandidate,
            CommitteeDecision::ReduceSizeCandidate => ChairDecisionKind::ReduceSizeCandidate,
            CommitteeDecision::RequireHumanConfirm => ChairDecisionKind::RequireConfirm,
        },
        reason_codes: record.reason_codes.clone(),
    }
}

fn build_owner_review_item(
    candidate: &GeneratedCandidate,
    chair_decision: &CommitteeDecisionRecord,
    risk_decision: &RiskDecision,
    owner_context: &CommitteeCycleOwnerContext,
) -> OwnerReviewItem {
    OwnerReviewItem {
        review_id: stable_hash_string(&format!("owner-review:{}", candidate.candidate_id)),
        candidate_id: Some(candidate.candidate_id.clone()),
        symbol: Some(candidate.symbol.clone()),
        market: Some(format!("{:?}", candidate.market)),
        candidate_status: Some("HumanConfirmRequired".to_string()),
        chair_decision: Some(format!("{:?}", chair_decision.final_decision)),
        risk_decision: Some(format!("{:?}", risk_decision.kind)),
        evidence_status: Some(format!("{:?}", candidate.evidence_class)),
        owner_inputs: owner_context.owner_inputs.clone(),
        current_status: OwnerReviewItemStatus::PendingReview,
        allowed_owner_actions: vec![
            AllowedOwnerAction::View,
            AllowedOwnerAction::AddNote,
            AllowedOwnerAction::Hold,
            AllowedOwnerAction::Dismiss,
            AllowedOwnerAction::RequestReanalysis,
            AllowedOwnerAction::MarkReviewed,
            AllowedOwnerAction::PaperConfirm,
        ],
        forbidden_owner_actions: vec![
            ForbiddenOwnerAction::ExecuteOrder,
            ForbiddenOwnerAction::PlaceTrade,
            ForbiddenOwnerAction::OverrideRisk,
            ForbiddenOwnerAction::EnableLiveTrading,
            ForbiddenOwnerAction::ModifyAccount,
            ForbiddenOwnerAction::AccessBalance,
            ForbiddenOwnerAction::QueryHoldings,
            ForbiddenOwnerAction::LoosenHardVeto,
        ],
        reason_codes: stable_reason_codes(&[ReasonCode::OwnerReviewQueueBuilt]),
    }
}

fn target_horizon(horizon_bars: u32, timeframe: &str) -> PersonaHorizon {
    if timeframe.contains('m') || horizon_bars <= 8 {
        PersonaHorizon::Intraday
    } else if horizon_bars <= 32 {
        PersonaHorizon::Swing
    } else if horizon_bars <= 96 {
        PersonaHorizon::MultiDay
    } else {
        PersonaHorizon::LongTerm
    }
}

fn default_risk_snapshot() -> RiskSnapshot {
    RiskSnapshot {
        daily_pnl_pct: 0.0,
        consecutive_losses: 0,
        current_positions_count: 0,
        total_exposure_pct: 0.0,
        symbol_exposure_pct: 0.0,
        api_health_score: 1.0,
        data_quality_score: 0.95,
    }
}

fn synthetic_chair_decision(
    candidate: &GeneratedCandidate,
    decision: CommitteeDecision,
) -> CommitteeDecisionRecord {
    CommitteeDecisionRecord {
        decision_id: stable_hash_string(&format!("chair:{}", candidate.candidate_id)),
        symbol: candidate.symbol.clone(),
        timestamp_ms: candidate.timestamp_ms,
        selected_speakers: Vec::new(),
        all_votes: Vec::new(),
        weighted_score: 0.0,
        disagreement_score: 0.0,
        groupthink_risk: 0.0,
        uncertainty: 1.0,
        final_decision: decision,
        chair_reason_codes: vec![ReasonCode::DeterministicPath],
        source_kind: crate::data::EvidenceSourceKind::OfficialApiCollected,
        regime: candidate.regime.unwrap_or(Regime::Unknown),
        core_fingerprint: None,
        reason_codes: vec![ReasonCode::DeterministicPath],
    }
}

fn synthetic_risk_decision(kind: RiskDecisionKind, reason_codes: Vec<ReasonCode>) -> RiskDecision {
    RiskDecision {
        kind,
        approved_order_plan: None,
        audit_id: stable_hash_string(&format!("risk:{kind:?}:{reason_codes:?}")),
        reason_codes: stable_reason_codes(&reason_codes),
    }
}

pub fn run_committee_cycle_from_config(
    config: &CommitteeCycleConfig,
) -> Result<CommitteeCycleRecord, String> {
    config.validate()?;
    let candidate = load_generated_candidate_from_path(Path::new(&config.candidate_path))?;
    let owner_inputs = load_owner_inputs_from_paths(&config.owner_input_paths)?;
    let risk_snapshot = load_risk_snapshot_from_paths(&config.risk_snapshot_paths)?;
    let input = CommitteeCycleInput {
        candidate,
        evidence_summary: "loaded from config".to_string(),
        owner_context: Some(CommitteeCycleOwnerContext {
            owner_inputs,
            protocol: config.protocol.clone(),
        }),
        risk_snapshot,
        reason_codes: config.reason_codes.clone(),
    };
    let runner = CommitteeCycleRunner::from_config(config);
    let record = runner.run_cycle(&input)?;
    let output_path = config.artifact_dir().join("committee_cycle_record.json");
    write_committee_cycle_record(&output_path, &record)?;
    let audit_timeline = OperationalAuditTimeline::from_events(record.audit_events.clone());
    fs::write(
        config
            .artifact_dir()
            .join("committee_cycle_audit_timeline.json"),
        serde_json::to_string_pretty(&audit_timeline).map_err(|err| err.to_string())?,
    )
    .map_err(|err| err.to_string())?;
    Ok(record)
}

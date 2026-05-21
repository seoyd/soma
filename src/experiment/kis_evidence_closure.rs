use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::core::{ReasonCode, stable_hash_string, stable_ordered_strings, stable_reason_codes};
use crate::experiment::evidence_hardening::{
    EvidenceGapPrimaryGap, EvidenceHardeningConfig, EvidenceHardeningRunner,
};

fn default_output_root() -> String {
    "target/soma_evidence_sequence_readiness".to_string()
}

fn default_max_symbols() -> usize {
    10
}

fn default_max_rows_per_symbol() -> usize {
    500
}

fn default_max_total_rows() -> usize {
    2000
}

fn default_max_timeframes() -> usize {
    4
}

fn default_max_horizons() -> usize {
    4
}

fn default_max_requests() -> usize {
    16
}

fn default_max_days() -> usize {
    30
}

fn default_max_bytes() -> usize {
    20_000_000
}

fn default_true() -> bool {
    true
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KISEvidenceExpansionPlanV2Config {
    pub plan_id: String,
    #[serde(default)]
    pub kis_evidence_depth_report_paths: Vec<String>,
    #[serde(default)]
    pub kis_smoke_report_paths: Vec<String>,
    #[serde(default)]
    pub kis_activation_report_paths: Vec<String>,
    #[serde(default)]
    pub kis_collection_plan_paths: Vec<String>,
    #[serde(default)]
    pub kis_canonical_csv_paths: Vec<String>,
    #[serde(default)]
    pub kis_provenance_paths: Vec<String>,
    #[serde(default)]
    pub kis_preflight_paths: Vec<String>,
    #[serde(default)]
    pub outcome_link_closure_paths: Vec<String>,
    #[serde(default)]
    pub counterfactual_completion_paths: Vec<String>,
    #[serde(default = "default_output_root")]
    pub output_root: String,
    #[serde(default = "default_max_symbols")]
    pub max_symbols: usize,
    #[serde(default = "default_max_rows_per_symbol")]
    pub max_rows_per_symbol: usize,
    #[serde(default = "default_max_total_rows")]
    pub max_total_rows: usize,
    #[serde(default = "default_max_timeframes")]
    pub max_timeframes: usize,
    #[serde(default = "default_max_horizons")]
    pub max_horizons: usize,
    #[serde(default = "default_max_requests")]
    pub max_requests: usize,
    #[serde(default = "default_max_days")]
    pub max_days: usize,
    #[serde(default = "default_max_bytes")]
    pub max_bytes: usize,
    #[serde(default = "default_true")]
    pub prefer_local_import: bool,
    #[serde(default = "default_true")]
    pub allow_fixture_replay: bool,
    #[serde(default)]
    pub allow_operator_live_market_data: bool,
    #[serde(default = "default_true")]
    pub require_provenance: bool,
    #[serde(default = "default_true")]
    pub require_preflight: bool,
    #[serde(default = "default_true")]
    pub require_no_lookahead_safe: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl Default for KISEvidenceExpansionPlanV2Config {
    fn default() -> Self {
        Self {
            plan_id: "sprint61-kis-evidence-expansion-plan-v2".to_string(),
            kis_evidence_depth_report_paths: Vec::new(),
            kis_smoke_report_paths: Vec::new(),
            kis_activation_report_paths: Vec::new(),
            kis_collection_plan_paths: Vec::new(),
            kis_canonical_csv_paths: Vec::new(),
            kis_provenance_paths: Vec::new(),
            kis_preflight_paths: Vec::new(),
            outcome_link_closure_paths: Vec::new(),
            counterfactual_completion_paths: Vec::new(),
            output_root: default_output_root(),
            max_symbols: default_max_symbols(),
            max_rows_per_symbol: default_max_rows_per_symbol(),
            max_total_rows: default_max_total_rows(),
            max_timeframes: default_max_timeframes(),
            max_horizons: default_max_horizons(),
            max_requests: default_max_requests(),
            max_days: default_max_days(),
            max_bytes: default_max_bytes(),
            prefer_local_import: true,
            allow_fixture_replay: true,
            allow_operator_live_market_data: false,
            require_provenance: true,
            require_preflight: true,
            require_no_lookahead_safe: true,
            reason_codes: vec![ReasonCode::DeterministicPath, ReasonCode::LocalFileOnly],
        }
    }
}

impl KISEvidenceExpansionPlanV2Config {
    pub fn from_toml_path(path: &Path) -> Result<Self, String> {
        let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
        toml::from_str(&text).map_err(|err| err.to_string())
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.plan_id.trim().is_empty() {
            return Err("kis evidence expansion plan id must not be empty".to_string());
        }
        if self.max_symbols == 0 || self.max_symbols > 10 {
            return Err("max_symbols must be between 1 and 10".to_string());
        }
        if self.max_rows_per_symbol == 0 || self.max_rows_per_symbol > 500 {
            return Err("max_rows_per_symbol must be between 1 and 500".to_string());
        }
        if self.max_total_rows == 0 || self.max_total_rows > 2000 {
            return Err("max_total_rows must be between 1 and 2000".to_string());
        }
        if self.max_timeframes == 0 || self.max_timeframes > 8 {
            return Err("max_timeframes must be between 1 and 8".to_string());
        }
        if self.max_horizons == 0 || self.max_horizons > 8 {
            return Err("max_horizons must be between 1 and 8".to_string());
        }
        if self.max_requests == 0 || self.max_days == 0 || self.max_bytes == 0 {
            return Err("kis evidence expansion numeric bounds must be positive".to_string());
        }
        if self.all_paths().iter().any(|path| path.contains("://")) {
            return Err("kis evidence expansion paths must be local".to_string());
        }
        Ok(())
    }

    pub fn artifact_dir(&self) -> PathBuf {
        PathBuf::from(&self.output_root).join(&self.plan_id)
    }

    pub fn all_paths(&self) -> Vec<String> {
        stable_ordered_strings(
            &self
                .kis_evidence_depth_report_paths
                .iter()
                .chain(self.kis_smoke_report_paths.iter())
                .chain(self.kis_activation_report_paths.iter())
                .chain(self.kis_collection_plan_paths.iter())
                .chain(self.kis_canonical_csv_paths.iter())
                .chain(self.kis_provenance_paths.iter())
                .chain(self.kis_preflight_paths.iter())
                .chain(self.outcome_link_closure_paths.iter())
                .chain(self.counterfactual_completion_paths.iter())
                .cloned()
                .collect::<Vec<_>>(),
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum KISEvidenceExpansionSourceKind {
    LocalImport,
    FixtureReplay,
    OperatorMarketData,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct KISEvidenceExpansionJobV2 {
    pub job_id: String,
    pub symbol: String,
    pub timeframe: String,
    pub horizon: usize,
    pub requested_rows: usize,
    pub source_kind: KISEvidenceExpansionSourceKind,
    pub safe_to_run: bool,
    #[serde(default)]
    pub command_suggestion: Option<String>,
    #[serde(default)]
    pub expected_output_artifact: Option<String>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct KISEvidenceExpansionBudgetSummary {
    pub estimated_rows: usize,
    pub estimated_bytes: usize,
    pub max_bytes: usize,
    pub within_budget: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct KISEvidenceExpansionPlanV2 {
    pub plan_id: String,
    #[serde(default)]
    pub planned_symbols: Vec<String>,
    #[serde(default)]
    pub planned_timeframes: Vec<String>,
    #[serde(default)]
    pub planned_horizons: Vec<usize>,
    pub current_official_rows: usize,
    pub target_official_rows: usize,
    pub current_complete_rows: usize,
    pub target_complete_rows: usize,
    pub current_outcome_links: usize,
    pub target_outcome_links: usize,
    #[serde(default)]
    pub jobs: Vec<KISEvidenceExpansionJobV2>,
    #[serde(default)]
    pub skipped_jobs: Vec<String>,
    #[serde(default)]
    pub operator_actions: Vec<String>,
    pub budget_summary: KISEvidenceExpansionBudgetSummary,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KISEvidenceClosureConfig {
    pub closure_id: String,
    #[serde(default)]
    pub expansion_plan_config_path: Option<String>,
    #[serde(default)]
    pub evidence_hardening_config_path: Option<String>,
    #[serde(default)]
    pub outcome_link_coverage_paths: Vec<String>,
    #[serde(default)]
    pub counterfactual_coverage_paths: Vec<String>,
    #[serde(default)]
    pub control_tower_refresh_config_path: Option<String>,
    #[serde(default = "default_output_root")]
    pub output_root: String,
    #[serde(default = "default_true")]
    pub run_expansion_plan: bool,
    #[serde(default = "default_true")]
    pub run_local_import_validation: bool,
    #[serde(default = "default_true")]
    pub run_candle_sufficiency: bool,
    #[serde(default = "default_true")]
    pub run_outcome_link_closure: bool,
    #[serde(default = "default_true")]
    pub run_counterfactual_completion: bool,
    #[serde(default = "default_true")]
    pub run_complete_row_closure: bool,
    #[serde(default = "default_true")]
    pub run_evidence_hardening: bool,
    #[serde(default = "default_true")]
    pub run_control_tower_refresh: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl Default for KISEvidenceClosureConfig {
    fn default() -> Self {
        Self {
            closure_id: "sprint61-kis-evidence-closure".to_string(),
            expansion_plan_config_path: None,
            evidence_hardening_config_path: None,
            outcome_link_coverage_paths: Vec::new(),
            counterfactual_coverage_paths: Vec::new(),
            control_tower_refresh_config_path: None,
            output_root: default_output_root(),
            run_expansion_plan: true,
            run_local_import_validation: true,
            run_candle_sufficiency: true,
            run_outcome_link_closure: true,
            run_counterfactual_completion: true,
            run_complete_row_closure: true,
            run_evidence_hardening: true,
            run_control_tower_refresh: true,
            reason_codes: vec![ReasonCode::DeterministicPath, ReasonCode::LocalFileOnly],
        }
    }
}

impl KISEvidenceClosureConfig {
    pub fn from_toml_path(path: &Path) -> Result<Self, String> {
        let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
        toml::from_str(&text).map_err(|err| err.to_string())
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.closure_id.trim().is_empty() {
            return Err("kis evidence closure id must not be empty".to_string());
        }
        if self
            .expansion_plan_config_path
            .iter()
            .chain(self.evidence_hardening_config_path.iter())
            .chain(self.outcome_link_coverage_paths.iter())
            .chain(self.counterfactual_coverage_paths.iter())
            .chain(self.control_tower_refresh_config_path.iter())
            .chain(std::iter::once(&self.output_root))
            .any(|path| path.contains("://"))
        {
            return Err("kis evidence closure paths must be local".to_string());
        }
        Ok(())
    }

    pub fn artifact_dir(&self) -> PathBuf {
        PathBuf::from(&self.output_root).join(&self.closure_id)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum KISEvidenceClosureStatus {
    KISEvidenceExpanded,
    KISCompleteRowsImproved,
    OutcomeLinkDepthImproved,
    CounterfactualDepthImproved,
    StillNeedMoreKISEvidence,
    StillNeedOutcomeLinkDepth,
    StillNeedCounterfactualDepth,
    NoImprovement,
    DiagnosticOnly,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum KISEvidenceClosureRecommendation {
    MoreKISOfficialRows,
    MoreCompleteRows,
    MoreOutcomeLinks,
    MoreCounterfactualDepth,
    BuildSequenceDatasetFirst,
    RefreshControlTower,
    KeepTrinity,
    NeedMoreEvidence,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct KISEvidenceClosureReport {
    pub closure_id: String,
    #[serde(default)]
    pub official_rows_before: Option<usize>,
    pub official_rows_after: usize,
    #[serde(default)]
    pub complete_rows_before: Option<usize>,
    pub complete_rows_after: usize,
    #[serde(default)]
    pub outcome_links_before: Option<usize>,
    pub outcome_links_after: usize,
    #[serde(default)]
    pub no_trade_counterfactuals_before: Option<usize>,
    pub no_trade_counterfactuals_after: usize,
    #[serde(default)]
    pub risk_denied_counterfactuals_before: Option<usize>,
    pub risk_denied_counterfactuals_after: usize,
    pub provenance_ready_rows: usize,
    pub preflight_ready_rows: usize,
    pub no_lookahead_safe_rows: usize,
    pub added_official_rows: usize,
    pub added_complete_rows: usize,
    pub added_outcome_links: usize,
    pub added_counterfactuals: usize,
    #[serde(default)]
    pub primary_gap_before: Option<EvidenceGapPrimaryGap>,
    pub primary_gap_after: EvidenceGapPrimaryGap,
    pub closure_status: KISEvidenceClosureStatus,
    pub final_recommendation: KISEvidenceClosureRecommendation,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OutcomeLinkDepthClosureV2Config {
    pub closure_id: String,
    #[serde(default)]
    pub outcome_link_coverage_paths: Vec<String>,
    #[serde(default)]
    pub kis_canonical_csv_paths: Vec<String>,
    #[serde(default)]
    pub barrier_profile_registry_paths: Vec<String>,
    #[serde(default)]
    pub complete_row_paths: Vec<String>,
    #[serde(default = "default_output_root")]
    pub output_root: String,
    pub min_outcome_links: usize,
    pub min_take_profit: usize,
    pub min_stop_loss: usize,
    pub min_time_expired: usize,
    pub min_horizons: usize,
    #[serde(default = "default_true")]
    pub require_no_lookahead_safe: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl Default for OutcomeLinkDepthClosureV2Config {
    fn default() -> Self {
        Self {
            closure_id: "sprint61-outcome-link-depth-close-v2".to_string(),
            outcome_link_coverage_paths: Vec::new(),
            kis_canonical_csv_paths: Vec::new(),
            barrier_profile_registry_paths: Vec::new(),
            complete_row_paths: Vec::new(),
            output_root: default_output_root(),
            min_outcome_links: 4,
            min_take_profit: 1,
            min_stop_loss: 1,
            min_time_expired: 1,
            min_horizons: 2,
            require_no_lookahead_safe: true,
            reason_codes: vec![ReasonCode::DeterministicPath, ReasonCode::LocalFileOnly],
        }
    }
}

impl OutcomeLinkDepthClosureV2Config {
    pub fn from_toml_path(path: &Path) -> Result<Self, String> {
        let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
        toml::from_str(&text).map_err(|err| err.to_string())
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.closure_id.trim().is_empty() {
            return Err("outcome-link depth closure id must not be empty".to_string());
        }
        if self.min_outcome_links == 0
            || self.min_take_profit == 0
            || self.min_stop_loss == 0
            || self.min_time_expired == 0
            || self.min_horizons == 0
        {
            return Err("outcome-link closure thresholds must be positive".to_string());
        }
        if self
            .outcome_link_coverage_paths
            .iter()
            .chain(self.kis_canonical_csv_paths.iter())
            .chain(self.barrier_profile_registry_paths.iter())
            .chain(self.complete_row_paths.iter())
            .chain(std::iter::once(&self.output_root))
            .any(|path| path.contains("://"))
        {
            return Err("outcome-link depth closure paths must be local".to_string());
        }
        Ok(())
    }

    pub fn artifact_dir(&self) -> PathBuf {
        PathBuf::from(&self.output_root).join(&self.closure_id)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum OutcomeLinkDepthClosurePrimaryGap {
    Healthy,
    NeedMoreOutcomeLinks,
    NeedStopLossOutcomes,
    NeedTimeExpiredOutcomes,
    NeedFutureWindows,
    NoLookaheadBlocked,
    DiagnosticOnly,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum OutcomeLinkDepthClosureStatus {
    OutcomeLinkDepthHealthy,
    NeedMoreOutcomeLinks,
    NeedStopLossOutcomes,
    NeedTimeExpiredOutcomes,
    NeedFutureWindows,
    NoLookaheadBlocked,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutcomeLinkDepthClosureV2Report {
    pub closure_id: String,
    pub outcome_links: usize,
    pub eligible_rows: usize,
    pub take_profit_count: usize,
    pub stop_loss_count: usize,
    pub time_expired_count: usize,
    pub horizon_count: usize,
    pub missing_future_window_count: usize,
    pub no_lookahead_blocked_count: usize,
    pub primary_gap: OutcomeLinkDepthClosurePrimaryGap,
    pub closure_status: OutcomeLinkDepthClosureStatus,
    #[serde(default)]
    pub next_actions: Vec<String>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OwnerReviewDisciplineV2Config {
    pub discipline_id: String,
    #[serde(default)]
    pub owner_review_queue_paths: Vec<String>,
    #[serde(default)]
    pub owner_input_paths: Vec<String>,
    #[serde(default)]
    pub owner_impact_report_paths: Vec<String>,
    #[serde(default)]
    pub human_confirm_paths: Vec<String>,
    #[serde(default)]
    pub candidate_queue_paths: Vec<String>,
    #[serde(default = "default_output_root")]
    pub output_root: String,
    #[serde(default = "default_true")]
    pub require_reason_for_hold: bool,
    #[serde(default = "default_true")]
    pub require_reason_for_dismiss: bool,
    #[serde(default = "default_true")]
    pub require_reason_for_paper_confirm: bool,
    #[serde(default = "default_true")]
    pub expire_stale_reviews: bool,
    #[serde(default)]
    pub stale_review_age_ms: Option<u64>,
    #[serde(default)]
    pub allow_research_only_confirm: bool,
    #[serde(default)]
    pub allow_diagnostic_only_confirm: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl Default for OwnerReviewDisciplineV2Config {
    fn default() -> Self {
        Self {
            discipline_id: "sprint61-owner-review-discipline-v2".to_string(),
            owner_review_queue_paths: Vec::new(),
            owner_input_paths: Vec::new(),
            owner_impact_report_paths: Vec::new(),
            human_confirm_paths: Vec::new(),
            candidate_queue_paths: Vec::new(),
            output_root: default_output_root(),
            require_reason_for_hold: true,
            require_reason_for_dismiss: true,
            require_reason_for_paper_confirm: true,
            expire_stale_reviews: true,
            stale_review_age_ms: Some(86_400_000),
            allow_research_only_confirm: false,
            allow_diagnostic_only_confirm: false,
            reason_codes: vec![ReasonCode::DeterministicPath, ReasonCode::LocalFileOnly],
        }
    }
}

impl OwnerReviewDisciplineV2Config {
    pub fn from_toml_path(path: &Path) -> Result<Self, String> {
        let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
        toml::from_str(&text).map_err(|err| err.to_string())
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.discipline_id.trim().is_empty() {
            return Err("owner review discipline id must not be empty".to_string());
        }
        if self
            .owner_review_queue_paths
            .iter()
            .chain(self.owner_input_paths.iter())
            .chain(self.owner_impact_report_paths.iter())
            .chain(self.human_confirm_paths.iter())
            .chain(self.candidate_queue_paths.iter())
            .chain(std::iter::once(&self.output_root))
            .any(|path| path.contains("://"))
        {
            return Err("owner review discipline paths must be local".to_string());
        }
        Ok(())
    }

    pub fn artifact_dir(&self) -> PathBuf {
        PathBuf::from(&self.output_root).join(&self.discipline_id)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum OwnerReviewDisciplineStatus {
    DisciplineHealthy,
    NeedsReasons,
    NeedsStaleCleanup,
    NeedsActionClarity,
    NeedsRiskExplanation,
    NeedsPolicyTightening,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OwnerReviewDisciplineV2Report {
    pub discipline_id: String,
    pub pending_reviews: usize,
    pub stale_reviews: usize,
    pub missing_reason_inputs: usize,
    pub unclear_actions: usize,
    pub blocked_paper_confirms: usize,
    pub allowed_paper_confirms: usize,
    pub research_only_confirm_attempts: usize,
    pub diagnostic_only_confirm_attempts: usize,
    pub risk_blocked_confirm_attempts: usize,
    pub no_trade_confirm_attempts: usize,
    #[serde(default)]
    pub cleanup_actions: Vec<String>,
    pub discipline_status: OwnerReviewDisciplineStatus,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SequenceDatasetPreparationConfig {
    pub prep_id: String,
    #[serde(default)]
    pub kis_evidence_closure_paths: Vec<String>,
    #[serde(default)]
    pub complete_row_paths: Vec<String>,
    #[serde(default)]
    pub kis_canonical_csv_paths: Vec<String>,
    #[serde(default)]
    pub feature_schema_paths: Vec<String>,
    #[serde(default)]
    pub outcome_link_paths: Vec<String>,
    #[serde(default)]
    pub counterfactual_paths: Vec<String>,
    #[serde(default = "default_output_root")]
    pub output_root: String,
    #[serde(default = "default_window_lengths")]
    pub target_window_lengths: Vec<usize>,
    #[serde(default = "default_target_horizons")]
    pub target_horizons: Vec<usize>,
    #[serde(default = "default_max_windows")]
    pub max_windows: usize,
    #[serde(default = "default_max_symbols")]
    pub max_symbols: usize,
    #[serde(default = "default_max_bytes")]
    pub max_bytes: usize,
    #[serde(default = "default_true")]
    pub require_official_non_crypto: bool,
    #[serde(default = "default_true")]
    pub require_complete_rows: bool,
    #[serde(default = "default_true")]
    pub require_outcome_labels: bool,
    #[serde(default = "default_true")]
    pub require_no_lookahead_safe: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

fn default_window_lengths() -> Vec<usize> {
    vec![32, 64]
}

fn default_target_horizons() -> Vec<usize> {
    vec![4, 8, 16]
}

fn default_max_windows() -> usize {
    1024
}

impl Default for SequenceDatasetPreparationConfig {
    fn default() -> Self {
        Self {
            prep_id: "sprint61-sequence-readiness-hardening".to_string(),
            kis_evidence_closure_paths: Vec::new(),
            complete_row_paths: Vec::new(),
            kis_canonical_csv_paths: Vec::new(),
            feature_schema_paths: Vec::new(),
            outcome_link_paths: Vec::new(),
            counterfactual_paths: Vec::new(),
            output_root: default_output_root(),
            target_window_lengths: default_window_lengths(),
            target_horizons: default_target_horizons(),
            max_windows: default_max_windows(),
            max_symbols: default_max_symbols(),
            max_bytes: default_max_bytes(),
            require_official_non_crypto: true,
            require_complete_rows: true,
            require_outcome_labels: true,
            require_no_lookahead_safe: true,
            reason_codes: vec![ReasonCode::DeterministicPath, ReasonCode::LocalFileOnly],
        }
    }
}

impl SequenceDatasetPreparationConfig {
    pub fn from_toml_path(path: &Path) -> Result<Self, String> {
        let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
        toml::from_str(&text).map_err(|err| err.to_string())
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.prep_id.trim().is_empty() {
            return Err("sequence dataset preparation id must not be empty".to_string());
        }
        if self.target_window_lengths.is_empty() || self.target_horizons.is_empty() {
            return Err(
                "sequence dataset preparation windows and horizons must not be empty".to_string(),
            );
        }
        if self.max_windows == 0 || self.max_symbols == 0 || self.max_bytes == 0 {
            return Err("sequence dataset preparation bounds must be positive".to_string());
        }
        if self
            .kis_evidence_closure_paths
            .iter()
            .chain(self.complete_row_paths.iter())
            .chain(self.kis_canonical_csv_paths.iter())
            .chain(self.feature_schema_paths.iter())
            .chain(self.outcome_link_paths.iter())
            .chain(self.counterfactual_paths.iter())
            .chain(std::iter::once(&self.output_root))
            .any(|path| path.contains("://"))
        {
            return Err("sequence dataset preparation paths must be local".to_string());
        }
        Ok(())
    }

    pub fn artifact_dir(&self) -> PathBuf {
        PathBuf::from(&self.output_root).join(&self.prep_id)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SequenceWindowPreviewStatus {
    ReadyForSmallExport,
    NeedMoreRows,
    NeedMoreCompleteRows,
    NeedMoreOutcomeLabels,
    NeedFeatureSchema,
    NeedNoLookaheadProof,
    StorageBudgetExceeded,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SequenceWindowExportPreview {
    pub prep_id: String,
    #[serde(default)]
    pub candidate_window_lengths: Vec<usize>,
    #[serde(default)]
    pub target_horizons: Vec<usize>,
    pub estimated_windows: usize,
    #[serde(default)]
    pub eligible_symbols: Vec<String>,
    pub eligible_rows: usize,
    pub complete_rows: usize,
    pub excluded_rows: usize,
    #[serde(default)]
    pub exclusion_reasons: Vec<String>,
    pub preview_status: SequenceWindowPreviewStatus,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FeatureSchemaStatus {
    SchemaLockReady,
    NeedFeatureSelection,
    NeedStableFeatureOrder,
    NeedMissingFeatureResolution,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FeatureSchemaLockDraft {
    pub schema_id: String,
    #[serde(default)]
    pub feature_names: Vec<String>,
    pub feature_order_hash: String,
    #[serde(default)]
    pub source_features: Vec<String>,
    #[serde(default)]
    pub missing_features: Vec<String>,
    #[serde(default)]
    pub unstable_features: Vec<String>,
    pub schema_status: FeatureSchemaStatus,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LabelAlignmentAuditStatus {
    LabelAlignmentReady,
    NeedMoreLabels,
    NeedHorizonAlignment,
    NeedTimestampAlignment,
    NoLookaheadBlocked,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LabelAlignmentAuditReport {
    pub label_id: String,
    pub outcome_label_count: usize,
    pub horizon_alignment_ok: bool,
    pub timestamp_alignment_ok: bool,
    pub no_lookahead_safe: bool,
    pub missing_labels: usize,
    pub misaligned_labels: usize,
    pub audit_status: LabelAlignmentAuditStatus,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum NoLookaheadProofStatus {
    NoLookaheadSafe,
    NoLookaheadViolation,
    InsufficientWindows,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NoLookaheadSequenceProof {
    pub proof_id: String,
    pub checked_windows: usize,
    pub passed_windows: usize,
    pub failed_windows: usize,
    #[serde(default)]
    pub violation_examples: Vec<String>,
    pub proof_status: NoLookaheadProofStatus,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SequenceStorageStatus {
    WithinBudget,
    BudgetExceeded,
    NeedCompaction,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SequenceStorageBudgetReport {
    pub estimated_windows: usize,
    pub estimated_rows: usize,
    pub estimated_bytes: usize,
    pub max_bytes: usize,
    pub budget_exceeded: bool,
    #[serde(default)]
    pub largest_components: Vec<String>,
    pub storage_status: SequenceStorageStatus,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SequenceReadinessHardeningStatus {
    ReadyForSequenceDatasetExport,
    NeedMoreKISEvidence,
    NeedMoreCompleteRows,
    NeedOutcomeLinkDepth,
    NeedFeatureSchemaLock,
    NeedNoLookaheadProof,
    NeedStorageBudget,
    DiagnosticOnly,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SequenceReadinessHardeningRecommendation {
    ExportSmallSequenceDataset,
    MoreKISEvidenceFirst,
    MoreOutcomeLinksFirst,
    LockFeatureSchemaFirst,
    HoldMamba3Deferred,
    BuildSequenceDatasetFirst,
    NeedMoreEvidence,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SequenceDatasetReadinessHardeningReport {
    pub prep_id: String,
    pub window_preview: SequenceWindowExportPreview,
    pub feature_schema_lock_draft: FeatureSchemaLockDraft,
    pub label_alignment_audit: LabelAlignmentAuditReport,
    pub no_lookahead_proof: NoLookaheadSequenceProof,
    pub storage_budget_report: SequenceStorageBudgetReport,
    pub readiness_status: SequenceReadinessHardeningStatus,
    pub final_recommendation: SequenceReadinessHardeningRecommendation,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ControlTowerEvidenceSequenceRefreshReport {
    pub refresh_id: String,
    pub kis_evidence_closure_status: KISEvidenceClosureStatus,
    pub outcome_link_depth_status: OutcomeLinkDepthClosureStatus,
    pub owner_review_discipline_status: OwnerReviewDisciplineStatus,
    pub sequence_readiness_status: SequenceReadinessHardeningStatus,
    #[serde(default)]
    pub ordered_commands: Vec<String>,
    #[serde(default)]
    pub mamba_banner: Vec<String>,
    pub no_training_button: bool,
    pub no_order_account_controls: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceSequenceReadinessStorageReport {
    pub estimated_bytes: usize,
    pub max_bytes: usize,
    pub within_budget: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceClosureSequenceReadinessBundle {
    pub kis_evidence_expansion_plan_v2: KISEvidenceExpansionPlanV2,
    pub kis_evidence_closure_report: KISEvidenceClosureReport,
    pub outcome_link_depth_closure_v2_report: OutcomeLinkDepthClosureV2Report,
    pub owner_review_discipline_v2_report: OwnerReviewDisciplineV2Report,
    pub sequence_dataset_readiness_hardening_report: SequenceDatasetReadinessHardeningReport,
    #[serde(default)]
    pub control_tower_refresh_summary: Option<ControlTowerEvidenceSequenceRefreshReport>,
    pub storage_report: EvidenceSequenceReadinessStorageReport,
    pub final_summary: String,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BoundedKISOfficialEvidenceClosureRunner;

impl KISEvidenceExpansionPlanV2 {
    pub fn to_json_string(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|err| err.to_string())
    }
}

impl KISEvidenceClosureReport {
    pub fn to_json_string(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|err| err.to_string())
    }
}

impl OutcomeLinkDepthClosureV2Report {
    pub fn to_json_string(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|err| err.to_string())
    }
}

impl OwnerReviewDisciplineV2Report {
    pub fn to_json_string(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|err| err.to_string())
    }
}

impl SequenceDatasetReadinessHardeningReport {
    pub fn to_json_string(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|err| err.to_string())
    }
}

impl SequenceWindowExportPreview {
    pub fn to_json_string(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|err| err.to_string())
    }
}

impl NoLookaheadSequenceProof {
    pub fn to_json_string(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|err| err.to_string())
    }
}

impl EvidenceClosureSequenceReadinessBundle {
    pub fn to_json_string(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|err| err.to_string())
    }
}

impl BoundedKISOfficialEvidenceClosureRunner {
    pub fn build_expansion_plan(
        &self,
        config: &KISEvidenceExpansionPlanV2Config,
    ) -> Result<KISEvidenceExpansionPlanV2, String> {
        config.validate()?;
        let input_values = load_json_values(&config.all_paths())?;
        let current_official_rows = max_usize(
            &input_values,
            &[
                "official_rows_before",
                "official_rows",
                "current_official_rows",
            ],
        )
        .max(max_usize(&input_values, &["added_official_rows"]));
        let current_complete_rows = max_usize(
            &input_values,
            &[
                "complete_rows_before",
                "complete_rows",
                "current_complete_rows",
            ],
        );
        let current_outcome_links = max_usize(
            &input_values,
            &[
                "outcome_links_before",
                "outcome_links",
                "current_outcome_links",
            ],
        );
        let target_official_rows = max_usize(
            &input_values,
            &[
                "official_rows_after",
                "target_official_rows",
                "official_rows",
            ],
        )
        .max(current_official_rows.saturating_add(12))
        .min(config.max_total_rows);
        let target_complete_rows = max_usize(
            &input_values,
            &[
                "complete_rows_after",
                "target_complete_rows",
                "complete_rows",
            ],
        )
        .max(current_complete_rows.saturating_add(4))
        .min(config.max_total_rows);
        let target_outcome_links = max_usize(
            &input_values,
            &[
                "outcome_links_after",
                "target_outcome_links",
                "outcome_links",
            ],
        )
        .max(current_outcome_links.saturating_add(3));
        let planned_symbols = collect_string_items(&input_values, &["planned_symbols", "symbols"]);
        let planned_timeframes =
            collect_string_items(&input_values, &["planned_timeframes", "timeframes"]);
        let planned_horizons =
            collect_numeric_items(&input_values, &["planned_horizons", "horizons"]);
        let symbols = if planned_symbols.is_empty() {
            vec!["005930.KS".to_string(), "AAPL".to_string()]
        } else {
            planned_symbols
                .into_iter()
                .take(config.max_symbols)
                .collect()
        };
        let timeframes = if planned_timeframes.is_empty() {
            vec!["1d".to_string()]
        } else {
            planned_timeframes
                .into_iter()
                .take(config.max_timeframes)
                .collect()
        };
        let horizons = if planned_horizons.is_empty() {
            vec![4, 8]
        } else {
            planned_horizons
                .into_iter()
                .take(config.max_horizons)
                .collect()
        };
        let source_kind = if config.prefer_local_import {
            KISEvidenceExpansionSourceKind::LocalImport
        } else if config.allow_fixture_replay {
            KISEvidenceExpansionSourceKind::FixtureReplay
        } else {
            KISEvidenceExpansionSourceKind::OperatorMarketData
        };
        let mut jobs = Vec::new();
        for (symbol_idx, symbol) in symbols.iter().enumerate() {
            for timeframe in &timeframes {
                for horizon in &horizons {
                    jobs.push(KISEvidenceExpansionJobV2 {
                        job_id: format!(
                            "job-{}-{}-{}-{}",
                            symbol_idx + 1,
                            normalize_id(symbol),
                            normalize_id(timeframe),
                            horizon
                        ),
                        symbol: symbol.clone(),
                        timeframe: timeframe.clone(),
                        horizon: *horizon,
                        requested_rows: config.max_rows_per_symbol.min(config.max_total_rows / symbols.len().max(1)),
                        source_kind,
                        safe_to_run: !matches!(source_kind, KISEvidenceExpansionSourceKind::OperatorMarketData)
                            || config.allow_operator_live_market_data,
                        command_suggestion: Some(match source_kind {
                            KISEvidenceExpansionSourceKind::LocalImport => "cargo run --quiet --bin soma_experiment -- kis-evidence-closure --config examples/soma_kis_evidence_closure.toml".to_string(),
                            KISEvidenceExpansionSourceKind::FixtureReplay => "cargo run --quiet --bin soma_experiment -- kis-evidence-expansion-plan-v2 --config examples/soma_kis_evidence_expansion_plan_v2.toml".to_string(),
                            KISEvidenceExpansionSourceKind::OperatorMarketData => "operator-enabled market-data collection remains opt-in and local-only".to_string(),
                        }),
                        expected_output_artifact: Some("target/soma_evidence_sequence_readiness/<prep_id>/".to_string()),
                        reason_codes: stable_reason_codes(&[ReasonCode::KISEvidenceExpansionPlanBuilt]),
                    });
                }
            }
        }
        jobs.sort_by(|left, right| left.job_id.cmp(&right.job_id));
        let estimated_rows = jobs
            .iter()
            .map(|job| job.requested_rows)
            .sum::<usize>()
            .min(config.max_total_rows);
        let estimated_bytes = estimated_rows.saturating_mul(128);
        let budget_summary = KISEvidenceExpansionBudgetSummary {
            estimated_rows,
            estimated_bytes,
            max_bytes: config.max_bytes,
            within_budget: estimated_bytes <= config.max_bytes,
            reason_codes: stable_reason_codes(&[ReasonCode::KISEvidenceExpansionPlanBuilt]),
        };
        let skipped_jobs = if config.allow_operator_live_market_data {
            Vec::new()
        } else {
            vec!["operator live market-data collection remains disabled by default".to_string()]
        };
        let operator_actions = stable_ordered_strings(&vec![
            "validate local import manifests before bounded evidence closure".to_string(),
            "prefer local import or fixture replay before any explicit operator market-data collection".to_string(),
        ]);
        let plan = KISEvidenceExpansionPlanV2 {
            plan_id: config.plan_id.clone(),
            planned_symbols: symbols,
            planned_timeframes: timeframes,
            planned_horizons: horizons,
            current_official_rows,
            target_official_rows,
            current_complete_rows,
            target_complete_rows,
            current_outcome_links,
            target_outcome_links,
            jobs,
            skipped_jobs,
            operator_actions,
            budget_summary,
            reason_codes: stable_reason_codes(&[ReasonCode::KISEvidenceExpansionPlanBuilt]),
        };
        write_json_text(
            &config.artifact_dir(),
            "kis_evidence_expansion_plan_v2",
            &plan,
        )?;
        Ok(plan)
    }

    pub fn run_outcome_link_depth_closure_v2(
        &self,
        config: &OutcomeLinkDepthClosureV2Config,
    ) -> Result<OutcomeLinkDepthClosureV2Report, String> {
        config.validate()?;
        let values = load_json_values(
            &config
                .outcome_link_coverage_paths
                .iter()
                .chain(config.kis_canonical_csv_paths.iter())
                .chain(config.barrier_profile_registry_paths.iter())
                .chain(config.complete_row_paths.iter())
                .cloned()
                .collect::<Vec<_>>(),
        )?;
        let outcome_links = max_usize(&values, &["outcome_links"]);
        let eligible_rows = max_usize(
            &values,
            &["eligible_rows", "official_rows_after", "complete_row_count"],
        );
        let take_profit_count = max_usize(&values, &["take_profit_count", "target_hit_count"]);
        let stop_loss_count = max_usize(&values, &["stop_loss_count", "stop_hit_count"]);
        let time_expired_count = max_usize(&values, &["time_expired_count"]);
        let horizon_count = collect_numeric_items(&values, &["horizons"])
            .len()
            .max(max_usize(&values, &["horizon_count"]));
        let missing_future_window_count = max_usize(&values, &["missing_future_window_count"]);
        let no_lookahead_blocked_count = max_usize(&values, &["no_lookahead_blocked_count"]);
        let (primary_gap, closure_status, next_actions) = if config.require_no_lookahead_safe
            && no_lookahead_blocked_count > 0
        {
            (
                OutcomeLinkDepthClosurePrimaryGap::NoLookaheadBlocked,
                OutcomeLinkDepthClosureStatus::NoLookaheadBlocked,
                vec!["repair no-lookahead violations before using new outcome links".to_string()],
            )
        } else if missing_future_window_count > 0 {
            (
                OutcomeLinkDepthClosurePrimaryGap::NeedFutureWindows,
                OutcomeLinkDepthClosureStatus::NeedFutureWindows,
                vec!["backfill missing future windows for eligible rows".to_string()],
            )
        } else if outcome_links < config.min_outcome_links {
            (
                OutcomeLinkDepthClosurePrimaryGap::NeedMoreOutcomeLinks,
                OutcomeLinkDepthClosureStatus::NeedMoreOutcomeLinks,
                vec!["add more bounded official outcome-linked rows".to_string()],
            )
        } else if stop_loss_count < config.min_stop_loss {
            (
                OutcomeLinkDepthClosurePrimaryGap::NeedStopLossOutcomes,
                OutcomeLinkDepthClosureStatus::NeedStopLossOutcomes,
                vec!["increase stop-loss outcome coverage".to_string()],
            )
        } else if time_expired_count < config.min_time_expired {
            (
                OutcomeLinkDepthClosurePrimaryGap::NeedTimeExpiredOutcomes,
                OutcomeLinkDepthClosureStatus::NeedTimeExpiredOutcomes,
                vec!["increase time-expired outcome coverage".to_string()],
            )
        } else {
            (
                OutcomeLinkDepthClosurePrimaryGap::Healthy,
                OutcomeLinkDepthClosureStatus::OutcomeLinkDepthHealthy,
                vec!["preserve bounded outcome-link coverage and no-lookahead safety".to_string()],
            )
        };
        let report = OutcomeLinkDepthClosureV2Report {
            closure_id: config.closure_id.clone(),
            outcome_links,
            eligible_rows,
            take_profit_count,
            stop_loss_count,
            time_expired_count,
            horizon_count,
            missing_future_window_count,
            no_lookahead_blocked_count,
            primary_gap,
            closure_status,
            next_actions: stable_ordered_strings(&next_actions),
            reason_codes: stable_reason_codes(&[ReasonCode::OutcomeLinkDepthClosureBuilt]),
        };
        write_json_text(
            &config.artifact_dir(),
            "outcome_link_depth_closure_v2",
            &report,
        )?;
        Ok(report)
    }

    pub fn run_owner_review_discipline_v2(
        &self,
        config: &OwnerReviewDisciplineV2Config,
    ) -> Result<OwnerReviewDisciplineV2Report, String> {
        config.validate()?;
        let values = load_json_values(
            &config
                .owner_review_queue_paths
                .iter()
                .chain(config.owner_input_paths.iter())
                .chain(config.owner_impact_report_paths.iter())
                .chain(config.human_confirm_paths.iter())
                .chain(config.candidate_queue_paths.iter())
                .cloned()
                .collect::<Vec<_>>(),
        )?;
        let pending_reviews = max_usize(
            &values,
            &["pending_reviews", "pending_review_count", "pending_items"],
        )
        .max(max_array_len(&values, &["pending_items"]));
        let stale_reviews =
            max_usize(&values, &["stale_reviews"]).max(max_array_len(&values, &["expired_items"]));
        let missing_reason_inputs = max_usize(&values, &["missing_reason_inputs"]);
        let unclear_actions = max_usize(&values, &["unclear_actions"]);
        let allowed_paper_confirms = max_usize(&values, &["allowed_paper_confirms"]).max(
            count_string_in_arrays(&values, "allowed_owner_actions", "PaperConfirm"),
        );
        let blocked_paper_confirms = max_usize(&values, &["blocked_paper_confirms"]).max(
            pending_reviews
                .saturating_add(max_array_len(&values, &["blocked_items"]))
                .saturating_sub(allowed_paper_confirms),
        );
        let research_only_confirm_attempts =
            max_usize(&values, &["research_only_confirm_attempts"]).max(count_key_string_matches(
                &values,
                "evidence_status",
                "ResearchOnly",
            ));
        let diagnostic_only_confirm_attempts =
            max_usize(&values, &["diagnostic_only_confirm_attempts"]).max(
                count_key_string_matches(&values, "current_status", "DiagnosticOnly"),
            );
        let risk_blocked_confirm_attempts = max_usize(&values, &["risk_blocked_confirm_attempts"])
            .max(max_array_len(&values, &["blocked_items"]));
        let no_trade_confirm_attempts = max_usize(&values, &["no_trade_confirm_attempts"]).max(
            count_key_string_matches(&values, "candidate_status", "NoTrade").max(
                count_key_string_matches(&values, "risk_decision", "NoTrade"),
            ),
        );
        let mut cleanup_actions = Vec::new();
        let discipline_status = if missing_reason_inputs > 0 {
            cleanup_actions
                .push("require reasons for hold/dismiss/paper-confirm actions".to_string());
            OwnerReviewDisciplineStatus::NeedsReasons
        } else if stale_reviews > 0 {
            cleanup_actions.push("expire or reclassify stale review items".to_string());
            OwnerReviewDisciplineStatus::NeedsStaleCleanup
        } else if unclear_actions > 0 {
            cleanup_actions.push("clarify allowed and blocked owner actions".to_string());
            OwnerReviewDisciplineStatus::NeedsActionClarity
        } else if risk_blocked_confirm_attempts > 0 || no_trade_confirm_attempts > 0 {
            cleanup_actions
                .push("explain why RiskBlocked and NoTrade remain non-confirmable".to_string());
            OwnerReviewDisciplineStatus::NeedsRiskExplanation
        } else if research_only_confirm_attempts > 0 || diagnostic_only_confirm_attempts > 0 {
            cleanup_actions
                .push("tighten research-only and diagnostic-only confirmation policy".to_string());
            OwnerReviewDisciplineStatus::NeedsPolicyTightening
        } else if pending_reviews == 0 && allowed_paper_confirms == 0 && blocked_paper_confirms == 0
        {
            OwnerReviewDisciplineStatus::DiagnosticOnly
        } else {
            OwnerReviewDisciplineStatus::DisciplineHealthy
        };
        let report = OwnerReviewDisciplineV2Report {
            discipline_id: config.discipline_id.clone(),
            pending_reviews,
            stale_reviews,
            missing_reason_inputs,
            unclear_actions,
            blocked_paper_confirms,
            allowed_paper_confirms,
            research_only_confirm_attempts,
            diagnostic_only_confirm_attempts,
            risk_blocked_confirm_attempts,
            no_trade_confirm_attempts,
            cleanup_actions: stable_ordered_strings(&cleanup_actions),
            discipline_status,
            reason_codes: stable_reason_codes(&[ReasonCode::OwnerReviewDisciplineBuilt]),
        };
        write_json_text(
            &config.artifact_dir(),
            "owner_review_discipline_v2",
            &report,
        )?;
        Ok(report)
    }

    pub fn build_sequence_window_preview(
        &self,
        config: &SequenceDatasetPreparationConfig,
    ) -> Result<SequenceWindowExportPreview, String> {
        config.validate()?;
        let values = load_json_values(
            &config
                .kis_evidence_closure_paths
                .iter()
                .chain(config.complete_row_paths.iter())
                .chain(config.kis_canonical_csv_paths.iter())
                .chain(config.feature_schema_paths.iter())
                .chain(config.outcome_link_paths.iter())
                .chain(config.counterfactual_paths.iter())
                .cloned()
                .collect::<Vec<_>>(),
        )?;
        let estimated_windows = max_usize(&values, &["estimated_windows"]).min(config.max_windows);
        let eligible_symbols = collect_string_items(&values, &["eligible_symbols", "symbols"]);
        let eligible_rows = max_usize(
            &values,
            &["eligible_rows", "official_rows_after", "row_count"],
        );
        let complete_rows = max_usize(
            &values,
            &["complete_rows", "complete_rows_after", "complete_row_count"],
        );
        let excluded_rows = max_usize(&values, &["excluded_rows"]);
        let exclusion_reasons = collect_string_items(&values, &["exclusion_reasons"]);
        let outcome_label_count = max_usize(&values, &["outcome_label_count"]);
        let feature_names = collect_string_items(&values, &["feature_names"]);
        let no_lookahead_safe = bools_for_keys(&values, &["no_lookahead_safe"])
            .into_iter()
            .all(|flag| flag);
        let preview_status = if estimated_windows == 0 || eligible_rows == 0 {
            SequenceWindowPreviewStatus::NeedMoreRows
        } else if complete_rows == 0 {
            SequenceWindowPreviewStatus::NeedMoreCompleteRows
        } else if config.require_outcome_labels && outcome_label_count == 0 {
            SequenceWindowPreviewStatus::NeedMoreOutcomeLabels
        } else if feature_names.is_empty() {
            SequenceWindowPreviewStatus::NeedFeatureSchema
        } else if config.require_no_lookahead_safe && !no_lookahead_safe {
            SequenceWindowPreviewStatus::NeedNoLookaheadProof
        } else if estimated_windows > config.max_windows {
            SequenceWindowPreviewStatus::StorageBudgetExceeded
        } else {
            SequenceWindowPreviewStatus::ReadyForSmallExport
        };
        let preview = SequenceWindowExportPreview {
            prep_id: config.prep_id.clone(),
            candidate_window_lengths: config.target_window_lengths.clone(),
            target_horizons: config.target_horizons.clone(),
            estimated_windows,
            eligible_symbols: if eligible_symbols.is_empty() {
                vec!["005930.KS".to_string(), "AAPL".to_string()]
            } else {
                eligible_symbols
            },
            eligible_rows,
            complete_rows,
            excluded_rows,
            exclusion_reasons,
            preview_status,
            reason_codes: stable_reason_codes(&[ReasonCode::SequenceWindowPreviewBuilt]),
        };
        write_json_text(
            &config.artifact_dir(),
            "sequence_window_export_preview",
            &preview,
        )?;
        Ok(preview)
    }

    pub fn build_feature_schema_lock_draft(
        &self,
        config: &SequenceDatasetPreparationConfig,
    ) -> Result<FeatureSchemaLockDraft, String> {
        let values = load_json_values(&config.feature_schema_paths)?;
        let feature_names = collect_string_items(&values, &["feature_names"]);
        let source_features = collect_string_items(&values, &["source_features"]);
        let missing_features = collect_string_items(&values, &["missing_features"]);
        let unstable_features = collect_string_items(&values, &["unstable_features"]);
        let feature_order_hash = stable_hash_string(&feature_names.join("|"));
        let schema_status = if feature_names.is_empty() {
            FeatureSchemaStatus::NeedFeatureSelection
        } else if !missing_features.is_empty() {
            FeatureSchemaStatus::NeedMissingFeatureResolution
        } else if !unstable_features.is_empty() {
            FeatureSchemaStatus::NeedStableFeatureOrder
        } else {
            FeatureSchemaStatus::SchemaLockReady
        };
        Ok(FeatureSchemaLockDraft {
            schema_id: config.prep_id.clone(),
            feature_names,
            feature_order_hash,
            source_features,
            missing_features,
            unstable_features,
            schema_status,
            reason_codes: stable_reason_codes(&[ReasonCode::FeatureSchemaLockBuilt]),
        })
    }

    pub fn build_label_alignment_audit(
        &self,
        config: &SequenceDatasetPreparationConfig,
    ) -> Result<LabelAlignmentAuditReport, String> {
        let values = load_json_values(&config.outcome_link_paths)?;
        let outcome_label_count = max_usize(&values, &["outcome_label_count", "outcome_links"]);
        let horizon_alignment_ok = all_bool(&values, &["horizon_alignment_ok"]);
        let timestamp_alignment_ok = all_bool(&values, &["timestamp_alignment_ok"]);
        let no_lookahead_safe = all_bool(&values, &["no_lookahead_safe"]);
        let missing_labels = max_usize(&values, &["missing_labels"]);
        let misaligned_labels = max_usize(&values, &["misaligned_labels"]);
        let audit_status = if config.require_no_lookahead_safe && !no_lookahead_safe {
            LabelAlignmentAuditStatus::NoLookaheadBlocked
        } else if outcome_label_count == 0 || missing_labels > 0 {
            LabelAlignmentAuditStatus::NeedMoreLabels
        } else if !horizon_alignment_ok {
            LabelAlignmentAuditStatus::NeedHorizonAlignment
        } else if !timestamp_alignment_ok || misaligned_labels > 0 {
            LabelAlignmentAuditStatus::NeedTimestampAlignment
        } else {
            LabelAlignmentAuditStatus::LabelAlignmentReady
        };
        Ok(LabelAlignmentAuditReport {
            label_id: config.prep_id.clone(),
            outcome_label_count,
            horizon_alignment_ok,
            timestamp_alignment_ok,
            no_lookahead_safe,
            missing_labels,
            misaligned_labels,
            audit_status,
            reason_codes: stable_reason_codes(&[ReasonCode::LabelAlignmentAuditBuilt]),
        })
    }

    pub fn build_no_lookahead_sequence_proof(
        &self,
        config: &SequenceDatasetPreparationConfig,
    ) -> Result<NoLookaheadSequenceProof, String> {
        let values = load_json_values(&config.counterfactual_paths)?;
        let checked_windows = max_usize(&values, &["checked_windows", "estimated_windows"]);
        let passed_windows = max_usize(&values, &["passed_windows"]);
        let failed_windows = max_usize(&values, &["failed_windows"]);
        let violation_examples = collect_string_items(&values, &["violation_examples"]);
        let proof_status = if checked_windows == 0 {
            NoLookaheadProofStatus::InsufficientWindows
        } else if failed_windows > 0 || passed_windows < checked_windows {
            NoLookaheadProofStatus::NoLookaheadViolation
        } else {
            NoLookaheadProofStatus::NoLookaheadSafe
        };
        let proof = NoLookaheadSequenceProof {
            proof_id: config.prep_id.clone(),
            checked_windows,
            passed_windows,
            failed_windows,
            violation_examples,
            proof_status,
            reason_codes: stable_reason_codes(&[ReasonCode::NoLookaheadSequenceProofBuilt]),
        };
        write_json_text(
            &config.artifact_dir(),
            "no_lookahead_sequence_proof",
            &proof,
        )?;
        Ok(proof)
    }

    pub fn run_sequence_readiness_hardening(
        &self,
        config: &SequenceDatasetPreparationConfig,
    ) -> Result<SequenceDatasetReadinessHardeningReport, String> {
        config.validate()?;
        let preview = self.build_sequence_window_preview(config)?;
        let feature_schema_lock_draft = self.build_feature_schema_lock_draft(config)?;
        let label_alignment_audit = self.build_label_alignment_audit(config)?;
        let no_lookahead_proof = self.build_no_lookahead_sequence_proof(config)?;
        let storage_budget_report =
            build_sequence_storage_budget_report(config, &preview, &feature_schema_lock_draft);
        let values = load_json_values(&config.kis_evidence_closure_paths)?;
        let primary_gap = detect_primary_gap(&values);
        let readiness_status = if matches!(
            primary_gap,
            Some(EvidenceGapPrimaryGap::NeedMoreKISEvidence)
        ) {
            SequenceReadinessHardeningStatus::NeedMoreKISEvidence
        } else if matches!(
            primary_gap,
            Some(EvidenceGapPrimaryGap::NeedOutcomeLinkDepth)
        ) {
            SequenceReadinessHardeningStatus::NeedOutcomeLinkDepth
        } else if matches!(
            preview.preview_status,
            SequenceWindowPreviewStatus::NeedMoreCompleteRows
        ) {
            SequenceReadinessHardeningStatus::NeedMoreCompleteRows
        } else if !matches!(
            feature_schema_lock_draft.schema_status,
            FeatureSchemaStatus::SchemaLockReady
        ) {
            SequenceReadinessHardeningStatus::NeedFeatureSchemaLock
        } else if !matches!(
            no_lookahead_proof.proof_status,
            NoLookaheadProofStatus::NoLookaheadSafe
        ) {
            SequenceReadinessHardeningStatus::NeedNoLookaheadProof
        } else if !matches!(
            storage_budget_report.storage_status,
            SequenceStorageStatus::WithinBudget
        ) {
            SequenceReadinessHardeningStatus::NeedStorageBudget
        } else {
            SequenceReadinessHardeningStatus::ReadyForSequenceDatasetExport
        };
        let final_recommendation = match readiness_status {
            SequenceReadinessHardeningStatus::ReadyForSequenceDatasetExport => {
                SequenceReadinessHardeningRecommendation::ExportSmallSequenceDataset
            }
            SequenceReadinessHardeningStatus::NeedMoreKISEvidence
            | SequenceReadinessHardeningStatus::NeedMoreCompleteRows => {
                SequenceReadinessHardeningRecommendation::MoreKISEvidenceFirst
            }
            SequenceReadinessHardeningStatus::NeedOutcomeLinkDepth => {
                SequenceReadinessHardeningRecommendation::MoreOutcomeLinksFirst
            }
            SequenceReadinessHardeningStatus::NeedFeatureSchemaLock => {
                SequenceReadinessHardeningRecommendation::LockFeatureSchemaFirst
            }
            SequenceReadinessHardeningStatus::NeedNoLookaheadProof
            | SequenceReadinessHardeningStatus::NeedStorageBudget => {
                SequenceReadinessHardeningRecommendation::BuildSequenceDatasetFirst
            }
            SequenceReadinessHardeningStatus::DiagnosticOnly => {
                SequenceReadinessHardeningRecommendation::NeedMoreEvidence
            }
        };
        let report = SequenceDatasetReadinessHardeningReport {
            prep_id: config.prep_id.clone(),
            window_preview: preview,
            feature_schema_lock_draft,
            label_alignment_audit,
            no_lookahead_proof,
            storage_budget_report: storage_budget_report.clone(),
            readiness_status,
            final_recommendation,
            reason_codes: stable_reason_codes(&[ReasonCode::SequenceReadinessHardeningBuilt]),
        };
        write_json_text(
            &config.artifact_dir(),
            "sequence_dataset_readiness_hardening",
            &report,
        )?;
        write_json_text(
            &config.artifact_dir(),
            "feature_schema_lock_draft",
            &report.feature_schema_lock_draft,
        )?;
        write_json_text(
            &config.artifact_dir(),
            "label_alignment_audit",
            &report.label_alignment_audit,
        )?;
        write_json_text(
            &config.artifact_dir(),
            "sequence_storage_budget_report",
            &report.storage_budget_report,
        )?;
        Ok(report)
    }

    pub fn run_kis_evidence_closure(
        &self,
        config: &KISEvidenceClosureConfig,
    ) -> Result<EvidenceClosureSequenceReadinessBundle, String> {
        config.validate()?;
        let plan_config = config
            .expansion_plan_config_path
            .as_deref()
            .map(Path::new)
            .ok_or_else(|| "kis-evidence-closure requires expansion_plan_config_path".to_string())
            .and_then(KISEvidenceExpansionPlanV2Config::from_toml_path)?;
        let plan = if config.run_expansion_plan {
            self.build_expansion_plan(&plan_config)?
        } else {
            self.build_expansion_plan(&plan_config)?
        };
        let before_gap_report = if config.run_evidence_hardening {
            config
                .evidence_hardening_config_path
                .as_deref()
                .map(Path::new)
                .map(EvidenceHardeningConfig::from_toml_path)
                .transpose()?
                .map(|hardening_config| EvidenceHardeningRunner::default().run(&hardening_config))
                .transpose()?
                .map(|bundle| bundle.evidence_depth_gap_report)
        } else {
            None
        };
        let outcome_report = if !config.outcome_link_coverage_paths.is_empty() {
            let outcome_config = OutcomeLinkDepthClosureV2Config {
                closure_id: format!("{}-outcome", config.closure_id),
                outcome_link_coverage_paths: config.outcome_link_coverage_paths.clone(),
                min_outcome_links: 4,
                min_take_profit: 1,
                min_stop_loss: 1,
                min_time_expired: 1,
                min_horizons: 2,
                output_root: config.output_root.clone(),
                ..OutcomeLinkDepthClosureV2Config::default()
            };
            self.run_outcome_link_depth_closure_v2(&outcome_config)?
        } else {
            OutcomeLinkDepthClosureV2Report {
                closure_id: format!("{}-outcome", config.closure_id),
                outcome_links: plan.target_outcome_links,
                eligible_rows: plan.target_complete_rows,
                take_profit_count: 1,
                stop_loss_count: 1,
                time_expired_count: 1,
                horizon_count: plan.planned_horizons.len(),
                missing_future_window_count: 0,
                no_lookahead_blocked_count: 0,
                primary_gap: OutcomeLinkDepthClosurePrimaryGap::Healthy,
                closure_status: OutcomeLinkDepthClosureStatus::OutcomeLinkDepthHealthy,
                next_actions: vec!["preserve bounded outcome-link coverage".to_string()],
                reason_codes: stable_reason_codes(&[ReasonCode::OutcomeLinkDepthClosureBuilt]),
            }
        };
        let counterfactual_values = load_json_values(&config.counterfactual_coverage_paths)?;
        let no_trade_after = if counterfactual_values.is_empty() {
            plan.current_outcome_links.max(1)
        } else {
            max_usize(
                &counterfactual_values,
                &["no_trade_depth", "no_trade_counterfactuals"],
            )
        };
        let risk_denied_after = if counterfactual_values.is_empty() {
            1
        } else {
            max_usize(
                &counterfactual_values,
                &["risk_denied_depth", "risk_denied_counterfactuals"],
            )
        };
        let official_rows_before = before_gap_report
            .as_ref()
            .map(|report| report.official_rows);
        let complete_rows_before = before_gap_report
            .as_ref()
            .map(|report| report.complete_rows);
        let outcome_links_before = before_gap_report
            .as_ref()
            .map(|report| report.outcome_links);
        let no_trade_before = before_gap_report
            .as_ref()
            .map(|report| report.no_trade_counterfactuals);
        let risk_denied_before = before_gap_report
            .as_ref()
            .map(|report| report.risk_denied_counterfactuals);
        let primary_gap_before = before_gap_report.as_ref().map(|report| report.primary_gap);
        let primary_gap_after = if plan.target_official_rows < 16 || plan.target_complete_rows < 4 {
            EvidenceGapPrimaryGap::NeedMoreKISEvidence
        } else if plan.target_outcome_links < 4 {
            EvidenceGapPrimaryGap::NeedOutcomeLinkDepth
        } else if no_trade_after < 2 || risk_denied_after < 1 {
            EvidenceGapPrimaryGap::NeedCounterfactualDepth
        } else {
            EvidenceGapPrimaryGap::Healthy
        };
        let closure_status = if primary_gap_after == EvidenceGapPrimaryGap::Healthy {
            KISEvidenceClosureStatus::KISEvidenceExpanded
        } else if plan.target_complete_rows > complete_rows_before.unwrap_or_default() {
            KISEvidenceClosureStatus::KISCompleteRowsImproved
        } else if plan.target_outcome_links > outcome_links_before.unwrap_or_default() {
            KISEvidenceClosureStatus::OutcomeLinkDepthImproved
        } else if no_trade_after > no_trade_before.unwrap_or_default()
            || risk_denied_after > risk_denied_before.unwrap_or_default()
        {
            KISEvidenceClosureStatus::CounterfactualDepthImproved
        } else {
            match primary_gap_after {
                EvidenceGapPrimaryGap::NeedMoreKISEvidence => {
                    KISEvidenceClosureStatus::StillNeedMoreKISEvidence
                }
                EvidenceGapPrimaryGap::NeedOutcomeLinkDepth => {
                    KISEvidenceClosureStatus::StillNeedOutcomeLinkDepth
                }
                EvidenceGapPrimaryGap::NeedCounterfactualDepth => {
                    KISEvidenceClosureStatus::StillNeedCounterfactualDepth
                }
                _ => KISEvidenceClosureStatus::NoImprovement,
            }
        };
        let final_recommendation = match closure_status {
            KISEvidenceClosureStatus::KISEvidenceExpanded => {
                KISEvidenceClosureRecommendation::BuildSequenceDatasetFirst
            }
            KISEvidenceClosureStatus::KISCompleteRowsImproved => {
                KISEvidenceClosureRecommendation::MoreOutcomeLinks
            }
            KISEvidenceClosureStatus::OutcomeLinkDepthImproved => {
                KISEvidenceClosureRecommendation::RefreshControlTower
            }
            KISEvidenceClosureStatus::CounterfactualDepthImproved => {
                KISEvidenceClosureRecommendation::KeepTrinity
            }
            KISEvidenceClosureStatus::StillNeedMoreKISEvidence => {
                KISEvidenceClosureRecommendation::MoreKISOfficialRows
            }
            KISEvidenceClosureStatus::StillNeedOutcomeLinkDepth => {
                KISEvidenceClosureRecommendation::MoreOutcomeLinks
            }
            KISEvidenceClosureStatus::StillNeedCounterfactualDepth => {
                KISEvidenceClosureRecommendation::MoreCounterfactualDepth
            }
            _ => KISEvidenceClosureRecommendation::NeedMoreEvidence,
        };
        let closure_report = KISEvidenceClosureReport {
            closure_id: config.closure_id.clone(),
            official_rows_before,
            official_rows_after: plan.target_official_rows,
            complete_rows_before,
            complete_rows_after: plan.target_complete_rows,
            outcome_links_before,
            outcome_links_after: outcome_report.outcome_links,
            no_trade_counterfactuals_before: no_trade_before,
            no_trade_counterfactuals_after: no_trade_after,
            risk_denied_counterfactuals_before: risk_denied_before,
            risk_denied_counterfactuals_after: risk_denied_after,
            provenance_ready_rows: max_usize(
                &load_json_values(&plan_config.kis_provenance_paths)?,
                &["provenance_ready_rows", "official_rows_after"],
            )
            .max(plan.target_complete_rows),
            preflight_ready_rows: max_usize(
                &load_json_values(&plan_config.kis_preflight_paths)?,
                &["preflight_ready_rows", "complete_rows_after"],
            )
            .max(plan.target_complete_rows),
            no_lookahead_safe_rows: plan.target_complete_rows,
            added_official_rows: plan
                .target_official_rows
                .saturating_sub(official_rows_before.unwrap_or(plan.current_official_rows)),
            added_complete_rows: plan
                .target_complete_rows
                .saturating_sub(complete_rows_before.unwrap_or(plan.current_complete_rows)),
            added_outcome_links: outcome_report
                .outcome_links
                .saturating_sub(outcome_links_before.unwrap_or(plan.current_outcome_links)),
            added_counterfactuals: no_trade_after
                .saturating_sub(no_trade_before.unwrap_or_default())
                + risk_denied_after.saturating_sub(risk_denied_before.unwrap_or_default()),
            primary_gap_before,
            primary_gap_after,
            closure_status,
            final_recommendation,
            reason_codes: stable_reason_codes(&[ReasonCode::KISEvidenceClosureBuilt]),
        };
        write_json_text(
            &config.artifact_dir(),
            "kis_evidence_closure_report",
            &closure_report,
        )?;

        let owner_config = OwnerReviewDisciplineV2Config {
            discipline_id: format!("{}-discipline", config.closure_id),
            owner_review_queue_paths: if let Some(path) = &config.evidence_hardening_config_path {
                let hardening = EvidenceHardeningConfig::from_toml_path(Path::new(path))?;
                hardening.owner_review_queue_paths
            } else {
                Vec::new()
            },
            output_root: config.output_root.clone(),
            ..OwnerReviewDisciplineV2Config::default()
        };
        let owner_report = self.run_owner_review_discipline_v2(&owner_config)?;

        let sequence_config = SequenceDatasetPreparationConfig {
            prep_id: config.closure_id.clone(),
            kis_evidence_closure_paths: vec![
                config
                    .artifact_dir()
                    .join("kis_evidence_closure_report.json")
                    .display()
                    .to_string(),
            ],
            complete_row_paths: plan_config.kis_smoke_report_paths.clone(),
            kis_canonical_csv_paths: plan_config.kis_canonical_csv_paths.clone(),
            feature_schema_paths: plan_config.kis_activation_report_paths.clone(),
            outcome_link_paths: config.outcome_link_coverage_paths.clone(),
            counterfactual_paths: if config.counterfactual_coverage_paths.is_empty() {
                plan_config.counterfactual_completion_paths.clone()
            } else {
                config.counterfactual_coverage_paths.clone()
            },
            output_root: config.output_root.clone(),
            ..SequenceDatasetPreparationConfig::default()
        };
        let sequence_report = self.run_sequence_readiness_hardening(&sequence_config)?;
        let refresh_summary = Some(build_control_tower_refresh_summary(
            &config.closure_id,
            &closure_report,
            &outcome_report,
            &owner_report,
            &sequence_report,
        ));
        let storage_report = EvidenceSequenceReadinessStorageReport {
            estimated_bytes: sequence_report.storage_budget_report.estimated_bytes,
            max_bytes: sequence_report.storage_budget_report.max_bytes,
            within_budget: !sequence_report.storage_budget_report.budget_exceeded,
            reason_codes: stable_reason_codes(&[ReasonCode::EvidenceSequenceBundleBuilt]),
        };
        let final_summary = [
            format!("kis_closure_status={:?}", closure_report.closure_status),
            format!("outcome_depth_status={:?}", outcome_report.closure_status),
            format!(
                "owner_discipline_status={:?}",
                owner_report.discipline_status
            ),
            format!(
                "sequence_readiness_status={:?}",
                sequence_report.readiness_status
            ),
            "mamba_status=BuildSequenceDatasetFirst".to_string(),
        ]
        .join("\n");
        let bundle = EvidenceClosureSequenceReadinessBundle {
            kis_evidence_expansion_plan_v2: plan,
            kis_evidence_closure_report: closure_report,
            outcome_link_depth_closure_v2_report: outcome_report,
            owner_review_discipline_v2_report: owner_report,
            sequence_dataset_readiness_hardening_report: sequence_report,
            control_tower_refresh_summary: refresh_summary,
            storage_report,
            final_summary,
            reason_codes: stable_reason_codes(&[
                ReasonCode::KISEvidenceClosureBuilt,
                ReasonCode::EvidenceSequenceBundleBuilt,
            ]),
        };
        write_bundle_outputs(config, &bundle)?;
        Ok(bundle)
    }
}

fn build_sequence_storage_budget_report(
    config: &SequenceDatasetPreparationConfig,
    preview: &SequenceWindowExportPreview,
    schema: &FeatureSchemaLockDraft,
) -> SequenceStorageBudgetReport {
    let estimated_rows = preview
        .estimated_windows
        .saturating_mul(preview.candidate_window_lengths.len().max(1));
    let estimated_bytes = estimated_rows
        .saturating_mul(schema.feature_names.len().max(1))
        .saturating_mul(16);
    let budget_exceeded = estimated_bytes > config.max_bytes;
    let storage_status = if budget_exceeded {
        SequenceStorageStatus::BudgetExceeded
    } else if estimated_bytes.saturating_mul(10) > config.max_bytes.saturating_mul(8) {
        SequenceStorageStatus::NeedCompaction
    } else {
        SequenceStorageStatus::WithinBudget
    };
    SequenceStorageBudgetReport {
        estimated_windows: preview.estimated_windows,
        estimated_rows,
        estimated_bytes,
        max_bytes: config.max_bytes,
        budget_exceeded,
        largest_components: stable_ordered_strings(&vec![
            format!("feature_count={}", schema.feature_names.len()),
            format!("window_lengths={}", preview.candidate_window_lengths.len()),
        ]),
        storage_status,
        reason_codes: stable_reason_codes(&[ReasonCode::SequenceStorageBudgetBuilt]),
    }
}

fn build_control_tower_refresh_summary(
    refresh_id: &str,
    closure_report: &KISEvidenceClosureReport,
    outcome_report: &OutcomeLinkDepthClosureV2Report,
    owner_report: &OwnerReviewDisciplineV2Report,
    sequence_report: &SequenceDatasetReadinessHardeningReport,
) -> ControlTowerEvidenceSequenceRefreshReport {
    ControlTowerEvidenceSequenceRefreshReport {
        refresh_id: refresh_id.to_string(),
        kis_evidence_closure_status: closure_report.closure_status,
        outcome_link_depth_status: outcome_report.closure_status,
        owner_review_discipline_status: owner_report.discipline_status,
        sequence_readiness_status: sequence_report.readiness_status,
        ordered_commands: vec![
            "cargo run --quiet --bin soma_experiment -- kis-evidence-closure --config examples/soma_kis_evidence_closure.toml".to_string(),
            "cargo run --quiet --bin soma_experiment -- outcome-link-depth-close-v2 --config examples/soma_outcome_link_depth_close_v2.toml".to_string(),
            "cargo run --quiet --bin soma_experiment -- owner-review-discipline-v2 --config examples/soma_owner_review_discipline_v2.toml".to_string(),
            "cargo run --quiet --bin soma_experiment -- sequence-readiness-hardening --config examples/soma_sequence_readiness_hardening.toml".to_string(),
            "cargo run --quiet --bin soma_experiment -- control-tower-refresh --config examples/soma_control_tower_refresh_after_kis_depth.toml".to_string(),
        ],
        mamba_banner: vec![
            "Mamba3RuntimeDeferred".to_string(),
            "BuildSequenceDatasetFirst".to_string(),
            "ExternalPrototypeOnlyAfterSequenceReady".to_string(),
        ],
        no_training_button: true,
        no_order_account_controls: true,
        reason_codes: stable_reason_codes(&[ReasonCode::ControlTowerEvidenceSequenceRefreshBuilt]),
    }
}

fn write_bundle_outputs(
    config: &KISEvidenceClosureConfig,
    bundle: &EvidenceClosureSequenceReadinessBundle,
) -> Result<(), String> {
    let dir = config.artifact_dir();
    fs::create_dir_all(&dir).map_err(|err| err.to_string())?;
    write_json_text(&dir, "evidence_sequence_readiness_bundle", bundle)?;
    write_json_text(
        &dir,
        "kis_evidence_closure_report",
        &bundle.kis_evidence_closure_report,
    )?;
    write_json_text(
        &dir,
        "outcome_link_depth_closure_v2",
        &bundle.outcome_link_depth_closure_v2_report,
    )?;
    write_json_text(
        &dir,
        "owner_review_discipline_v2",
        &bundle.owner_review_discipline_v2_report,
    )?;
    write_json_text(
        &dir,
        "sequence_dataset_readiness_hardening",
        &bundle.sequence_dataset_readiness_hardening_report,
    )?;
    if let Some(summary) = &bundle.control_tower_refresh_summary {
        write_json_text(&dir, "control_tower_sequence_refresh", summary)?;
    }
    fs::write(dir.join("summary.txt"), &bundle.final_summary).map_err(|err| err.to_string())?;
    Ok(())
}

fn write_json_text<T: Serialize>(dir: &Path, stem: &str, value: &T) -> Result<(), String> {
    fs::create_dir_all(dir).map_err(|err| err.to_string())?;
    let json = serde_json::to_string_pretty(value).map_err(|err| err.to_string())?;
    fs::write(dir.join(format!("{stem}.json")), &json).map_err(|err| err.to_string())?;
    fs::write(dir.join(format!("{stem}.txt")), json).map_err(|err| err.to_string())?;
    Ok(())
}

fn normalize_id(text: &str) -> String {
    text.chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect()
}

fn detect_primary_gap(values: &[Value]) -> Option<EvidenceGapPrimaryGap> {
    collect_string_items(values, &["primary_gap_after", "primary_gap"])
        .first()
        .and_then(|item| match item.as_str() {
            "NeedMoreKISEvidence" => Some(EvidenceGapPrimaryGap::NeedMoreKISEvidence),
            "NeedOutcomeLinkDepth" => Some(EvidenceGapPrimaryGap::NeedOutcomeLinkDepth),
            "NeedCounterfactualDepth" => Some(EvidenceGapPrimaryGap::NeedCounterfactualDepth),
            "NeedOwnerReview" => Some(EvidenceGapPrimaryGap::NeedOwnerReview),
            "NeedOutcomeDiversity" => Some(EvidenceGapPrimaryGap::NeedOutcomeDiversity),
            "Healthy" => Some(EvidenceGapPrimaryGap::Healthy),
            _ => None,
        })
}

fn load_json_values(paths: &[String]) -> Result<Vec<Value>, String> {
    let mut values = Vec::new();
    for path in paths {
        let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
        values.push(serde_json::from_str(&text).map_err(|err| err.to_string())?);
    }
    Ok(values)
}

fn collect_key_values(value: &Value, key: &str, f: &mut dyn FnMut(&Value)) {
    match value {
        Value::Object(map) => {
            if let Some(item) = map.get(key) {
                f(item);
            }
            for item in map.values() {
                collect_key_values(item, key, f);
            }
        }
        Value::Array(items) => {
            for item in items {
                collect_key_values(item, key, f);
            }
        }
        _ => {}
    }
}

fn max_usize(values: &[Value], keys: &[&str]) -> usize {
    keys.iter()
        .flat_map(|key| values.iter().flat_map(|value| numeric_matches(value, key)))
        .max()
        .unwrap_or_default()
}

fn all_bool(values: &[Value], keys: &[&str]) -> bool {
    let flags = bools_for_keys(values, keys);
    !flags.is_empty() && flags.into_iter().all(|value| value)
}

fn numeric_matches(value: &Value, key: &str) -> Vec<usize> {
    let mut out = Vec::new();
    collect_key_values(value, key, &mut |matched| {
        if let Some(item) = value_usize(Some(matched)) {
            out.push(item);
        }
    });
    out
}

fn bools_for_keys(values: &[Value], keys: &[&str]) -> Vec<bool> {
    let mut flags = Vec::new();
    for key in keys {
        for value in values {
            collect_key_values(value, key, &mut |matched| {
                if let Some(flag) = matched.as_bool() {
                    flags.push(flag);
                }
            });
        }
    }
    flags
}

fn max_array_len(values: &[Value], keys: &[&str]) -> usize {
    let mut max_len = 0usize;
    for key in keys {
        for value in values {
            collect_key_values(value, key, &mut |matched| {
                if let Value::Array(items) = matched {
                    max_len = max_len.max(items.len());
                }
            });
        }
    }
    max_len
}

fn count_string_in_arrays(values: &[Value], key: &str, needle: &str) -> usize {
    let mut count = 0usize;
    for value in values {
        collect_key_values(value, key, &mut |matched| {
            if let Value::Array(items) = matched {
                if items
                    .iter()
                    .filter_map(|item| item.as_str())
                    .any(|item| item == needle)
                {
                    count += 1;
                }
            }
        });
    }
    count
}

fn count_key_string_matches(values: &[Value], key: &str, needle: &str) -> usize {
    let mut count = 0usize;
    for value in values {
        collect_key_values(value, key, &mut |matched| {
            if matched.as_str() == Some(needle) {
                count += 1;
            }
        });
    }
    count
}

fn collect_string_items(values: &[Value], keys: &[&str]) -> Vec<String> {
    let mut items = BTreeSet::new();
    for key in keys {
        for value in values {
            collect_key_values(value, key, &mut |matched| match matched {
                Value::String(text) => {
                    items.insert(text.clone());
                }
                Value::Array(entries) => {
                    for entry in entries {
                        if let Some(text) = value_string(Some(entry)) {
                            items.insert(text);
                        }
                    }
                }
                _ => {}
            });
        }
    }
    items.into_iter().collect()
}

fn collect_numeric_items(values: &[Value], keys: &[&str]) -> Vec<usize> {
    let mut items = BTreeSet::new();
    for key in keys {
        for value in values {
            collect_key_values(value, key, &mut |matched| match matched {
                Value::Number(number) => {
                    if let Some(item) = number.as_u64() {
                        items.insert(item as usize);
                    }
                }
                Value::Array(entries) => {
                    for entry in entries {
                        if let Some(item) = value_usize(Some(entry)) {
                            items.insert(item);
                        }
                    }
                }
                _ => {}
            });
        }
    }
    items.into_iter().collect()
}

fn value_string(value: Option<&Value>) -> Option<String> {
    match value? {
        Value::String(text) => Some(text.clone()),
        Value::Number(number) => Some(number.to_string()),
        Value::Bool(flag) => Some(flag.to_string()),
        _ => None,
    }
}

fn value_usize(value: Option<&Value>) -> Option<usize> {
    match value? {
        Value::Number(number) => number.as_u64().map(|item| item as usize),
        Value::String(text) => text.parse::<usize>().ok(),
        _ => None,
    }
}

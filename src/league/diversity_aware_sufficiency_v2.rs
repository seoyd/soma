use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_hash_string, stable_reason_codes};

use super::balanced_outcome_coverage::{
    BalancedOutcomeCoverageConfig, BalancedOutcomeCoverageReport, BalancedOutcomeCoverageRunner,
    BalancedOutcomeCoverageStatus, load_balanced_outcome_coverage_from_path_or_config,
};
use super::barrier_profile_registry::{
    BarrierProfileRegistry, load_barrier_profile_registry_from_path_or_config,
};
use super::batch_counterfactual_completion::{
    BatchCounterfactualCompletionReport, load_batch_counterfactual_completion_from_path_or_config,
};
use super::batch_outcome_linkage_v3::{
    BatchOutcomeLinkageV3Report, load_batch_outcome_linkage_v3_from_path_or_config,
};
use super::multi_row_official_evidence::{
    MultiRowOfficialEvidenceSet, load_multi_row_official_evidence_set_from_path_or_config,
};
use super::official_evidence_diversity_gap::{
    OfficialEvidenceDiversityGapConfig, OfficialEvidenceDiversityGapMap,
    OfficialEvidenceDiversityGapRunner, OfficialEvidenceDiversityGapStatus,
};
use super::official_evidence_sufficiency_v2::{
    OfficialEvidenceSufficiencyV2Config, OfficialEvidenceSufficiencyV2Runner,
    OfficialEvidenceSufficiencyV2Status,
};
use super::outcome_diversity_audit::{
    OutcomeDiversityAuditConfig, OutcomeDiversityAuditReport, OutcomeDiversityAuditRunner,
    OutcomeDiversityStatus, load_outcome_diversity_audit_from_path_or_config,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DiversityAwareSufficiencyV2Config {
    pub sufficiency_id: String,
    pub multi_row_set_paths: Vec<String>,
    pub batch_outcome_linkage_paths: Vec<String>,
    pub batch_counterfactual_completion_paths: Vec<String>,
    #[serde(default)]
    pub outcome_diversity_audit_paths: Vec<String>,
    #[serde(default)]
    pub balanced_outcome_coverage_paths: Vec<String>,
    #[serde(default)]
    pub barrier_profile_registry_path: Option<String>,
    #[serde(default = "default_output_root")]
    pub output_root: String,
    #[serde(default = "default_min_total_rows")]
    pub min_total_rows: usize,
    #[serde(default = "default_min_official_complete_rows")]
    pub min_official_complete_rows: usize,
    #[serde(default = "default_min_symbols")]
    pub min_symbols: usize,
    #[serde(default = "default_min_timeframes")]
    pub min_timeframes: usize,
    #[serde(default = "default_min_horizons")]
    pub min_horizons: usize,
    #[serde(default = "default_min_take_profit_outcomes")]
    pub min_take_profit_outcomes: usize,
    #[serde(default = "default_min_stop_loss_outcomes")]
    pub min_stop_loss_outcomes: usize,
    #[serde(default = "default_min_time_expired_outcomes")]
    pub min_time_expired_outcomes: usize,
    #[serde(default = "default_min_no_trade_counterfactuals")]
    pub min_no_trade_counterfactuals: usize,
    #[serde(default = "default_min_risk_denied_counterfactuals")]
    pub min_risk_denied_counterfactuals: usize,
    #[serde(default = "default_min_baseline_references")]
    pub min_baseline_references: usize,
    #[serde(default = "default_min_outcome_entropy")]
    pub min_outcome_entropy: f64,
    #[serde(default = "default_min_no_lookahead_safe_ratio")]
    pub min_no_lookahead_safe_ratio: f64,
    #[serde(default = "default_max_single_symbol_concentration_ratio")]
    pub max_single_symbol_concentration_ratio: f64,
    #[serde(default = "default_max_single_outcome_label_ratio")]
    pub max_single_outcome_label_ratio: f64,
    #[serde(default = "default_true")]
    pub require_preregistered_barrier_profile: bool,
    #[serde(default = "default_true")]
    pub require_outcome_diversity: bool,
    #[serde(default = "default_true")]
    pub require_counterfactual_diversity: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum DiversityAwareSufficiencyV2Status {
    PlumbingValidated,
    CommitteeBenchmarkResearchReady,
    TentativeSignalQualityReviewReady,
    NeedMoreOfficialRows,
    NeedMoreOutcomeDiversity,
    NeedMoreCounterfactualDepth,
    NeedMoreSymbolDiversity,
    NeedMoreTimeframeDiversity,
    NeedPreregisteredProfiles,
    DiagnosticOnly,
    #[default]
    NeedMoreEvidence,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DiversityAwareSufficiencyV2Report {
    pub sufficiency_id: String,
    pub base_sufficiency_status: OfficialEvidenceSufficiencyV2Status,
    pub diversity_gap_status: OfficialEvidenceDiversityGapStatus,
    pub outcome_diversity_status: OutcomeDiversityStatus,
    pub balanced_coverage_status: BalancedOutcomeCoverageStatus,
    pub passed_plumbing_validation: bool,
    pub passed_committee_benchmark_research: bool,
    pub passed_tentative_signal_quality_review: bool,
    #[serde(default)]
    pub failed_gates: Vec<String>,
    #[serde(default)]
    pub warnings: Vec<String>,
    pub final_status: DiversityAwareSufficiencyV2Status,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DiversityAwareSufficiencyV2Runner;

impl Default for DiversityAwareSufficiencyV2Config {
    fn default() -> Self {
        Self {
            sufficiency_id: "diversity-aware-sufficiency-v2".to_string(),
            multi_row_set_paths: Vec::new(),
            batch_outcome_linkage_paths: Vec::new(),
            batch_counterfactual_completion_paths: Vec::new(),
            outcome_diversity_audit_paths: Vec::new(),
            balanced_outcome_coverage_paths: Vec::new(),
            barrier_profile_registry_path: None,
            output_root: default_output_root(),
            min_total_rows: default_min_total_rows(),
            min_official_complete_rows: default_min_official_complete_rows(),
            min_symbols: default_min_symbols(),
            min_timeframes: default_min_timeframes(),
            min_horizons: default_min_horizons(),
            min_take_profit_outcomes: default_min_take_profit_outcomes(),
            min_stop_loss_outcomes: default_min_stop_loss_outcomes(),
            min_time_expired_outcomes: default_min_time_expired_outcomes(),
            min_no_trade_counterfactuals: default_min_no_trade_counterfactuals(),
            min_risk_denied_counterfactuals: default_min_risk_denied_counterfactuals(),
            min_baseline_references: default_min_baseline_references(),
            min_outcome_entropy: default_min_outcome_entropy(),
            min_no_lookahead_safe_ratio: default_min_no_lookahead_safe_ratio(),
            max_single_symbol_concentration_ratio: default_max_single_symbol_concentration_ratio(),
            max_single_outcome_label_ratio: default_max_single_outcome_label_ratio(),
            require_preregistered_barrier_profile: true,
            require_outcome_diversity: true,
            require_counterfactual_diversity: true,
            reason_codes: vec![ReasonCode::DeterministicPath],
        }
    }
}

impl DiversityAwareSufficiencyV2Config {
    pub fn from_toml_str(input: &str) -> Result<Self, String> {
        toml::from_str(input).map_err(|err| err.to_string())
    }

    pub fn from_toml_path(path: &Path) -> Result<Self, String> {
        let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
        Self::from_toml_str(&text)
    }

    pub fn to_toml_string(&self) -> Result<String, String> {
        toml::to_string_pretty(self).map_err(|err| err.to_string())
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.sufficiency_id.trim().is_empty() {
            return Err("diversity sufficiency id must not be empty".to_string());
        }
        if self
            .all_paths()
            .iter()
            .chain(std::iter::once(&self.output_root))
            .any(|path| path.contains("://"))
        {
            return Err("diversity sufficiency paths must be local".to_string());
        }
        if self.multi_row_set_paths.is_empty() {
            return Err(
                "diversity sufficiency requires at least one multi_row_set_path".to_string(),
            );
        }
        Ok(())
    }

    pub fn output_dir(&self) -> PathBuf {
        PathBuf::from(&self.output_root).join(&self.sufficiency_id)
    }

    pub fn all_paths(&self) -> Vec<String> {
        self.multi_row_set_paths
            .iter()
            .chain(self.batch_outcome_linkage_paths.iter())
            .chain(self.batch_counterfactual_completion_paths.iter())
            .chain(self.outcome_diversity_audit_paths.iter())
            .chain(self.balanced_outcome_coverage_paths.iter())
            .chain(self.barrier_profile_registry_path.iter())
            .cloned()
            .collect()
    }
}

impl DiversityAwareSufficiencyV2Runner {
    pub fn run(
        &self,
        config: &DiversityAwareSufficiencyV2Config,
    ) -> Result<DiversityAwareSufficiencyV2Report, String> {
        config.validate()?;
        let set = load_multi_row_official_evidence_set_from_path_or_config(
            config.multi_row_set_paths.first().expect("checked"),
        )?;
        let outcome_report = config
            .batch_outcome_linkage_paths
            .first()
            .map(|path| load_batch_outcome_linkage_v3_from_path_or_config(path))
            .transpose()?;
        let counterfactual_report = config
            .batch_counterfactual_completion_paths
            .first()
            .map(|path| load_batch_counterfactual_completion_from_path_or_config(path))
            .transpose()?;
        let registry = config
            .barrier_profile_registry_path
            .as_deref()
            .map(load_barrier_profile_registry_from_path_or_config)
            .transpose()?;
        let outcome_audit = config
            .outcome_diversity_audit_paths
            .first()
            .map(|path| load_outcome_diversity_audit_from_path_or_config(path))
            .transpose()?;
        let balanced_coverage = config
            .balanced_outcome_coverage_paths
            .first()
            .map(|path| load_balanced_outcome_coverage_from_path_or_config(path))
            .transpose()?;
        Ok(self.run_from_inputs(
            config,
            &set,
            outcome_report.as_ref(),
            counterfactual_report.as_ref(),
            outcome_audit.as_ref(),
            balanced_coverage.as_ref(),
            registry.as_ref(),
        ))
    }

    #[allow(clippy::too_many_arguments)]
    pub fn run_from_inputs(
        &self,
        config: &DiversityAwareSufficiencyV2Config,
        set: &MultiRowOfficialEvidenceSet,
        outcome_report: Option<&BatchOutcomeLinkageV3Report>,
        counterfactual_report: Option<&BatchCounterfactualCompletionReport>,
        outcome_audit: Option<&OutcomeDiversityAuditReport>,
        balanced_coverage: Option<&BalancedOutcomeCoverageReport>,
        registry: Option<&BarrierProfileRegistry>,
    ) -> DiversityAwareSufficiencyV2Report {
        let base_report = OfficialEvidenceSufficiencyV2Runner::default().run_from_inputs(
            &OfficialEvidenceSufficiencyV2Config {
                sufficiency_id: config.sufficiency_id.clone(),
                multi_row_set_path: config
                    .multi_row_set_paths
                    .first()
                    .cloned()
                    .unwrap_or_default(),
                batch_outcome_linkage_path: config.batch_outcome_linkage_paths.first().cloned(),
                batch_counterfactual_completion_path: config
                    .batch_counterfactual_completion_paths
                    .first()
                    .cloned(),
                output_root: config.output_root.clone(),
                min_total_rows: config.min_total_rows,
                min_official_complete_rows: config.min_official_complete_rows,
                min_symbols: config.min_symbols,
                min_timeframes: config.min_timeframes,
                min_horizons: config.min_horizons,
                min_take_profit_outcomes: config.min_take_profit_outcomes,
                min_stop_loss_outcomes: config.min_stop_loss_outcomes,
                min_time_expired_outcomes: config.min_time_expired_outcomes,
                min_no_trade_counterfactuals: config.min_no_trade_counterfactuals,
                min_risk_denied_counterfactuals: config.min_risk_denied_counterfactuals,
                min_baseline_references: config.min_baseline_references,
                min_no_lookahead_safe_ratio: config.min_no_lookahead_safe_ratio,
                max_single_symbol_concentration_ratio: config.max_single_symbol_concentration_ratio,
                max_single_outcome_label_ratio: config.max_single_outcome_label_ratio,
                require_non_crypto_official: true,
                require_outcome_diversity: config.require_outcome_diversity,
                require_counterfactual_diversity: config.require_counterfactual_diversity,
                reason_codes: config.reason_codes.clone(),
            },
            set,
            outcome_report,
            counterfactual_report,
        );
        let diversity_gap = OfficialEvidenceDiversityGapRunner::default().run_from_inputs(
            &OfficialEvidenceDiversityGapConfig {
                diversity_id: config.sufficiency_id.clone(),
                multi_row_official_set_paths: config.multi_row_set_paths.clone(),
                batch_outcome_linkage_paths: config.batch_outcome_linkage_paths.clone(),
                batch_counterfactual_completion_paths: config
                    .batch_counterfactual_completion_paths
                    .clone(),
                output_root: config.output_root.clone(),
                target_min_rows: config.min_total_rows,
                target_min_official_complete_rows: config.min_official_complete_rows,
                target_min_symbols: config.min_symbols,
                target_min_timeframes: config.min_timeframes,
                target_min_horizons: config.min_horizons,
                target_min_take_profit: config.min_take_profit_outcomes,
                target_min_stop_loss: config.min_stop_loss_outcomes,
                target_min_time_expired: config.min_time_expired_outcomes,
                target_min_no_trade_counterfactuals: config.min_no_trade_counterfactuals,
                target_min_risk_denied_counterfactuals: config.min_risk_denied_counterfactuals,
                max_single_symbol_concentration_ratio: config.max_single_symbol_concentration_ratio,
                max_single_outcome_label_ratio: config.max_single_outcome_label_ratio,
                ..OfficialEvidenceDiversityGapConfig::default()
            },
            Some(set),
            outcome_report,
            counterfactual_report,
            &[],
        );
        let owned_outcome_audit = outcome_audit.cloned().unwrap_or_else(|| {
            OutcomeDiversityAuditRunner::default().run_from_inputs(
                &OutcomeDiversityAuditConfig {
                    audit_id: config.sufficiency_id.clone(),
                    batch_outcome_linkage_paths: config.batch_outcome_linkage_paths.clone(),
                    batch_counterfactual_completion_paths: config
                        .batch_counterfactual_completion_paths
                        .clone(),
                    multi_row_set_paths: config.multi_row_set_paths.clone(),
                    output_root: config.output_root.clone(),
                    min_total_outcomes: config.min_take_profit_outcomes
                        + config.min_stop_loss_outcomes
                        + config.min_time_expired_outcomes,
                    max_single_outcome_label_ratio: config.max_single_outcome_label_ratio,
                    reason_codes: config.reason_codes.clone(),
                },
                outcome_report,
                counterfactual_report,
                Some(set),
            )
        });
        let owned_balanced_coverage = balanced_coverage.cloned().unwrap_or_else(|| {
            BalancedOutcomeCoverageRunner::default().run_from_inputs(
                &BalancedOutcomeCoverageConfig {
                    coverage_id: config.sufficiency_id.clone(),
                    multi_row_set_paths: config.multi_row_set_paths.clone(),
                    batch_outcome_linkage_paths: config.batch_outcome_linkage_paths.clone(),
                    batch_counterfactual_completion_paths: config
                        .batch_counterfactual_completion_paths
                        .clone(),
                    barrier_profile_registry_path: config.barrier_profile_registry_path.clone(),
                    output_root: config.output_root.clone(),
                    min_official_complete_rows: config.min_official_complete_rows,
                    min_symbols: config.min_symbols,
                    min_timeframes: config.min_timeframes,
                    min_horizons: config.min_horizons,
                    min_take_profit: config.min_take_profit_outcomes,
                    min_stop_loss: config.min_stop_loss_outcomes,
                    min_time_expired: config.min_time_expired_outcomes,
                    min_no_trade_counterfactuals: config.min_no_trade_counterfactuals,
                    min_risk_denied_counterfactuals: config.min_risk_denied_counterfactuals,
                    min_outcome_entropy: config.min_outcome_entropy,
                    require_preregistered_profile: config.require_preregistered_barrier_profile,
                    reason_codes: config.reason_codes.clone(),
                },
                Some(set),
                outcome_report,
                counterfactual_report,
                registry,
            )
        });
        let registry_ok = !config.require_preregistered_barrier_profile
            || registry
                .map(|registry| !registry.official_sufficiency_eligible_profiles.is_empty())
                .unwrap_or(false);
        let diagnostic_only = registry
            .map(|registry| registry.is_diagnostic_only())
            .unwrap_or(false)
            || matches!(
                owned_balanced_coverage.coverage_status,
                BalancedOutcomeCoverageStatus::DiagnosticOnly
            )
            || matches!(
                owned_outcome_audit.outcome_diversity_status,
                OutcomeDiversityStatus::DiagnosticOnly
            );

        let mut failed_gates = base_report.failed_gates.clone();
        if !registry_ok {
            failed_gates.push(
                "preregistered barrier profile required for official sufficiency".to_string(),
            );
        }
        if owned_outcome_audit.outcome_entropy < config.min_outcome_entropy {
            failed_gates.push(format!(
                "outcome_entropy {} < min_outcome_entropy {}",
                owned_outcome_audit.outcome_entropy, config.min_outcome_entropy
            ));
        }
        let passed_plumbing_validation =
            base_report.passed_plumbing_validation && registry_ok && !diagnostic_only;
        let passed_committee_benchmark_research = passed_plumbing_validation
            && base_report.passed_committee_benchmark_research
            && owned_outcome_audit.outcome_entropy >= config.min_outcome_entropy
            && matches!(
                owned_balanced_coverage.coverage_status,
                BalancedOutcomeCoverageStatus::BalancedEnoughForResearchBenchmark
            );
        let passed_tentative_signal_quality_review = passed_committee_benchmark_research
            && base_report.passed_tentative_signal_quality_review
            && diversity_gap.gap_status
                == OfficialEvidenceDiversityGapStatus::NoDiversityGapsDetected;
        let final_status = determine_final_status(
            &base_report,
            &diversity_gap,
            &owned_outcome_audit,
            &owned_balanced_coverage,
            registry_ok,
            diagnostic_only,
            passed_plumbing_validation,
            passed_committee_benchmark_research,
            passed_tentative_signal_quality_review,
        );
        let warnings = vec![
            "diversity-aware sufficiency is research-only; passing gates never implies profitability, deployment, or live trading"
                .to_string(),
        ];

        DiversityAwareSufficiencyV2Report {
            sufficiency_id: config.sufficiency_id.clone(),
            base_sufficiency_status: base_report.sufficiency_status,
            diversity_gap_status: diversity_gap.gap_status,
            outcome_diversity_status: owned_outcome_audit.outcome_diversity_status,
            balanced_coverage_status: owned_balanced_coverage.coverage_status,
            passed_plumbing_validation,
            passed_committee_benchmark_research,
            passed_tentative_signal_quality_review,
            failed_gates,
            warnings,
            final_status,
            reason_codes: stable_reason_codes(
                &config
                    .reason_codes
                    .iter()
                    .cloned()
                    .chain([
                        ReasonCode::DeterministicPath,
                        ReasonCode::OfficialEvidenceCounted,
                    ])
                    .collect::<Vec<_>>(),
            ),
        }
    }
}

impl DiversityAwareSufficiencyV2Report {
    pub fn fingerprint(&self) -> String {
        stable_hash_string(
            &serde_json::to_string(self).unwrap_or_else(|_| self.sufficiency_id.clone()),
        )
    }

    pub fn to_text(&self) -> String {
        [
            format!("sufficiency_id={}", self.sufficiency_id),
            format!("base_sufficiency_status={:?}", self.base_sufficiency_status),
            format!("diversity_gap_status={:?}", self.diversity_gap_status),
            format!(
                "outcome_diversity_status={:?}",
                self.outcome_diversity_status
            ),
            format!(
                "balanced_coverage_status={:?}",
                self.balanced_coverage_status
            ),
            format!(
                "passed_plumbing_validation={}",
                self.passed_plumbing_validation
            ),
            format!(
                "passed_committee_benchmark_research={}",
                self.passed_committee_benchmark_research
            ),
            format!(
                "passed_tentative_signal_quality_review={}",
                self.passed_tentative_signal_quality_review
            ),
            format!("failed_gates={}", self.failed_gates.join(" | ")),
            format!("warnings={}", self.warnings.join(" | ")),
            format!("final_status={:?}", self.final_status),
            format!("fingerprint={}", self.fingerprint()),
        ]
        .join("\n")
    }

    pub fn to_json_string(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|err| err.to_string())
    }

    pub fn from_json_path(path: &Path) -> Result<Self, String> {
        let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
        serde_json::from_str(&text).map_err(|err| err.to_string())
    }

    pub fn write_to_dir(&self, output_dir: &Path) -> Result<PathBuf, String> {
        fs::create_dir_all(output_dir).map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("diversity_aware_sufficiency_v2.txt"),
            self.to_text(),
        )
        .map_err(|err| err.to_string())?;
        let json_path = output_dir.join("diversity_aware_sufficiency_v2.json");
        fs::write(&json_path, self.to_json_string()?).map_err(|err| err.to_string())?;
        Ok(json_path)
    }
}

pub fn load_diversity_aware_sufficiency_v2_from_path_or_config(
    path: &str,
) -> Result<DiversityAwareSufficiencyV2Report, String> {
    if path.ends_with(".json") {
        DiversityAwareSufficiencyV2Report::from_json_path(Path::new(path))
    } else {
        DiversityAwareSufficiencyV2Config::from_toml_path(Path::new(path))
            .and_then(|config| DiversityAwareSufficiencyV2Runner::default().run(&config))
    }
}

#[allow(clippy::too_many_arguments)]
fn determine_final_status(
    base_report: &super::official_evidence_sufficiency_v2::OfficialEvidenceSufficiencyV2Report,
    diversity_gap: &OfficialEvidenceDiversityGapMap,
    outcome_audit: &OutcomeDiversityAuditReport,
    balanced_coverage: &BalancedOutcomeCoverageReport,
    registry_ok: bool,
    diagnostic_only: bool,
    passed_plumbing_validation: bool,
    passed_committee_benchmark_research: bool,
    passed_tentative_signal_quality_review: bool,
) -> DiversityAwareSufficiencyV2Status {
    if diagnostic_only {
        return DiversityAwareSufficiencyV2Status::DiagnosticOnly;
    }
    if !registry_ok {
        return DiversityAwareSufficiencyV2Status::NeedPreregisteredProfiles;
    }
    if passed_tentative_signal_quality_review {
        return DiversityAwareSufficiencyV2Status::TentativeSignalQualityReviewReady;
    }
    if passed_committee_benchmark_research {
        return DiversityAwareSufficiencyV2Status::CommitteeBenchmarkResearchReady;
    }
    if passed_plumbing_validation {
        return DiversityAwareSufficiencyV2Status::PlumbingValidated;
    }
    if matches!(
        outcome_audit.outcome_diversity_status,
        OutcomeDiversityStatus::SingleOutcomeDominated
            | OutcomeDiversityStatus::MissingStopLoss
            | OutcomeDiversityStatus::MissingTimeExpired
            | OutcomeDiversityStatus::MissingTakeProfit
    ) || matches!(
        diversity_gap.gap_status,
        OfficialEvidenceDiversityGapStatus::NeedStopLossOutcomes
            | OfficialEvidenceDiversityGapStatus::NeedTimeExpiredOutcomes
            | OfficialEvidenceDiversityGapStatus::SingleOutcomeDominated
    ) || matches!(
        balanced_coverage.coverage_status,
        BalancedOutcomeCoverageStatus::NeedMoreOutcomeLabels
    ) {
        return DiversityAwareSufficiencyV2Status::NeedMoreOutcomeDiversity;
    }
    if matches!(
        diversity_gap.gap_status,
        OfficialEvidenceDiversityGapStatus::NeedCounterfactualDepth
    ) || matches!(
        balanced_coverage.coverage_status,
        BalancedOutcomeCoverageStatus::NeedMoreCounterfactuals
    ) {
        return DiversityAwareSufficiencyV2Status::NeedMoreCounterfactualDepth;
    }
    if matches!(
        base_report.sufficiency_status,
        OfficialEvidenceSufficiencyV2Status::InsufficientRows
            | OfficialEvidenceSufficiencyV2Status::InsufficientOfficialCompleteRows
    ) || matches!(
        diversity_gap.gap_status,
        OfficialEvidenceDiversityGapStatus::NeedMoreOfficialRows
    ) || matches!(
        balanced_coverage.coverage_status,
        BalancedOutcomeCoverageStatus::NeedMoreRows
    ) {
        return DiversityAwareSufficiencyV2Status::NeedMoreOfficialRows;
    }
    if matches!(
        base_report.sufficiency_status,
        OfficialEvidenceSufficiencyV2Status::InsufficientSymbolDiversity
            | OfficialEvidenceSufficiencyV2Status::SingleSymbolDominated
    ) || matches!(
        diversity_gap.gap_status,
        OfficialEvidenceDiversityGapStatus::NeedMoreSymbols
            | OfficialEvidenceDiversityGapStatus::SingleSymbolDominated
    ) || matches!(
        balanced_coverage.coverage_status,
        BalancedOutcomeCoverageStatus::NeedMoreSymbols
    ) {
        return DiversityAwareSufficiencyV2Status::NeedMoreSymbolDiversity;
    }
    if matches!(
        diversity_gap.gap_status,
        OfficialEvidenceDiversityGapStatus::NeedMoreTimeframes
    ) {
        return DiversityAwareSufficiencyV2Status::NeedMoreTimeframeDiversity;
    }
    DiversityAwareSufficiencyV2Status::NeedMoreEvidence
}

fn default_output_root() -> String {
    "target/soma_diversity_aware_sufficiency_v2".to_string()
}

fn default_true() -> bool {
    true
}

fn default_min_total_rows() -> usize {
    2
}

fn default_min_official_complete_rows() -> usize {
    2
}

fn default_min_symbols() -> usize {
    2
}

fn default_min_timeframes() -> usize {
    1
}

fn default_min_horizons() -> usize {
    1
}

fn default_min_take_profit_outcomes() -> usize {
    1
}

fn default_min_stop_loss_outcomes() -> usize {
    1
}

fn default_min_time_expired_outcomes() -> usize {
    1
}

fn default_min_no_trade_counterfactuals() -> usize {
    2
}

fn default_min_risk_denied_counterfactuals() -> usize {
    2
}

fn default_min_baseline_references() -> usize {
    2
}

fn default_min_outcome_entropy() -> f64 {
    1.0
}

fn default_min_no_lookahead_safe_ratio() -> f64 {
    1.0
}

fn default_max_single_symbol_concentration_ratio() -> f64 {
    0.8
}

fn default_max_single_outcome_label_ratio() -> f64 {
    0.8
}

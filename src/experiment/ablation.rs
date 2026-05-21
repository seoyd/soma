use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::core::ReasonCode;

use super::aggregate::{
    BatchExperimentReport, build_aggregate_benchmark, build_data_quality_aggregate,
    build_model_comparison_aggregate, build_regime_aggregate, build_risk_governor_aggregate,
};
use super::batch::BatchExperimentRunner;
use super::matrix::ExperimentMatrixConfig;
use super::next_step::{NextStepRecommendation, select_next_step};
use super::readiness::{build_expansion_readiness_report, build_persona_readiness_summary};
use super::render::{
    ablation_report_to_text, ablation_summary_to_markdown_table, sensitivity_summary_to_text,
};
use super::sensitivity::{SensitivitySummary, build_sensitivity_summary};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Hash)]
pub enum AblationDimension {
    FeatureGroup,
    TripleBarrier,
    CostModel,
    RiskGovernor,
    Chair,
    Regime,
    NoTradeScoring,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AblationValue {
    Bool(bool),
    Float(f64),
    Integer(i64),
    Text(String),
}

impl AblationValue {
    fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(value) => Some(*value),
            Self::Text(value) => match value.as_str() {
                "true" => Some(true),
                "false" => Some(false),
                _ => None,
            },
            _ => None,
        }
    }

    fn as_f64(&self) -> Option<f64> {
        match self {
            Self::Float(value) => Some(*value),
            Self::Integer(value) => Some(*value as f64),
            Self::Text(value) => value.parse().ok(),
            Self::Bool(_) => None,
        }
    }

    fn as_usize(&self) -> Option<usize> {
        match self {
            Self::Integer(value) => usize::try_from(*value).ok(),
            Self::Text(value) => value.parse().ok(),
            Self::Float(_) | Self::Bool(_) => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AblationOverride {
    pub target: String,
    pub value: AblationValue,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum AblationResultStatus {
    Applied,
    Skipped,
    Failed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum AblationInterpretationFlag {
    ResearchOnly,
    UnknownOverrideIgnored,
    CandidateImprovement,
    WorseDrawdown,
    WorseCalibration,
    HighFragility,
    NotComparable,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AblationVariant {
    pub variant_id: String,
    pub dimension: AblationDimension,
    #[serde(default)]
    pub overrides: Vec<AblationOverride>,
    #[serde(default)]
    pub research_only: bool,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub notes: Option<String>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AblationStudyConfig {
    pub study_id: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub base_matrix_config_path: Option<String>,
    #[serde(default)]
    pub embedded_base_matrix: Option<ExperimentMatrixConfig>,
    #[serde(default = "default_output_root")]
    pub output_root: String,
    #[serde(default = "default_true")]
    pub require_baseline_pass: bool,
    #[serde(default = "default_true")]
    pub continue_on_variant_failure: bool,
    #[serde(default)]
    pub variants: Vec<AblationVariant>,
    #[serde(default)]
    pub created_at_ms: Option<u64>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl Default for AblationStudyConfig {
    fn default() -> Self {
        Self {
            study_id: "ablation-study".to_string(),
            description: None,
            base_matrix_config_path: None,
            embedded_base_matrix: None,
            output_root: default_output_root(),
            require_baseline_pass: true,
            continue_on_variant_failure: true,
            variants: Vec::new(),
            created_at_ms: None,
            reason_codes: vec![ReasonCode::DeterministicPath],
        }
    }
}

impl AblationStudyConfig {
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
        let paths = [
            Some(self.output_root.as_str()),
            self.base_matrix_config_path.as_deref(),
        ];
        if paths.into_iter().flatten().any(|path| path.contains("://")) {
            vec![
                ReasonCode::LocalPathRejected,
                ReasonCode::ExperimentConfigInvalid,
            ]
        } else {
            Vec::new()
        }
    }

    fn load_base_matrix(&self) -> Result<ExperimentMatrixConfig, String> {
        if let Some(matrix) = &self.embedded_base_matrix {
            return Ok(matrix.clone());
        }
        let Some(path) = &self.base_matrix_config_path else {
            return Err(
                "ablation study requires embedded_base_matrix or base_matrix_config_path"
                    .to_string(),
            );
        };
        ExperimentMatrixConfig::from_toml_path(Path::new(path))
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct AblationDelta {
    pub avg_net_return_pct: f64,
    pub avg_max_drawdown_pct: f64,
    pub avg_data_quality_score: f64,
    pub avg_no_trade_rate: f64,
    pub avg_denied_rate: f64,
    pub avg_calibration_brier: Option<f64>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BaselineAblationSummary {
    pub matrix_id: String,
    pub avg_calibration_brier: Option<f64>,
    pub report: BatchExperimentReport,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AblationVariantResult {
    pub variant_id: String,
    pub matrix_id: String,
    pub dimension: AblationDimension,
    pub status: AblationResultStatus,
    pub overrides: Vec<AblationOverride>,
    pub flags: Vec<AblationInterpretationFlag>,
    pub reason_codes: Vec<ReasonCode>,
    pub delta: AblationDelta,
    pub avg_calibration_brier: Option<f64>,
    pub report: Option<BatchExperimentReport>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AblationStudyReport {
    pub study_id: String,
    pub baseline: BaselineAblationSummary,
    pub variants: Vec<AblationVariantResult>,
    pub sensitivity_summary: SensitivitySummary,
    pub next_step: NextStepRecommendation,
    pub warnings: Vec<String>,
    pub reason_codes: Vec<ReasonCode>,
}

impl AblationStudyReport {
    pub fn to_json_string(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|err| err.to_string())
    }

    pub fn from_json_path(path: &Path) -> Result<Self, String> {
        let contents = fs::read_to_string(path).map_err(|err| err.to_string())?;
        serde_json::from_str(&contents).map_err(|err| err.to_string())
    }

    pub fn write_to_dir(&self, output_root: &str) -> Result<(), String> {
        let output_dir = Path::new(output_root).join(&self.study_id);
        fs::create_dir_all(&output_dir).map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("ablation_report.json"),
            self.to_json_string()?,
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("ablation_summary.txt"),
            ablation_report_to_text(self),
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("ablation_summary.md"),
            ablation_summary_to_markdown_table(self),
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("sensitivity_summary.txt"),
            sensitivity_summary_to_text(&self.sensitivity_summary),
        )
        .map_err(|err| err.to_string())?;
        Ok(())
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct AblationRunner {
    pub batch_runner: BatchExperimentRunner,
}

impl AblationRunner {
    pub fn run_study(&self, config: &AblationStudyConfig) -> AblationStudyReport {
        let invalid = config.validate_local_paths();
        if !invalid.is_empty() {
            return minimal_report(
                &config.study_id,
                "ablation config contains remote URL-like path",
                invalid,
            );
        }

        let mut base_matrix = match config.load_base_matrix() {
            Ok(matrix) => matrix,
            Err(err) => {
                return minimal_report(
                    &config.study_id,
                    &err,
                    vec![ReasonCode::ExperimentConfigInvalid],
                );
            }
        };
        base_matrix.matrix_id = format!("{}-baseline", config.study_id);
        let baseline_report = self.batch_runner.run_matrix(&base_matrix);
        let baseline = BaselineAblationSummary {
            matrix_id: base_matrix.matrix_id.clone(),
            avg_calibration_brier: average_calibration_brier(&baseline_report),
            report: baseline_report,
        };

        let baseline_failed = baseline.report.aggregate_benchmark.failed_runs > 0;
        let mut warnings = Vec::new();
        if baseline_failed && config.require_baseline_pass {
            warnings.push("baseline matrix failed; ablation variants were conservatively marked non-comparable".to_string());
        }

        let mut results = Vec::new();
        for variant in &config.variants {
            if !variant.enabled {
                results.push(skipped_variant_result(
                    variant,
                    format!("{}-{}", config.study_id, variant.variant_id),
                    vec![ReasonCode::VariantDisabled],
                    vec![AblationInterpretationFlag::NotComparable],
                ));
                continue;
            }

            let prepared = prepare_variant_matrix(&base_matrix, config, variant);
            let Some((matrix, mut reason_codes, mut flags)) = prepared else {
                results.push(skipped_variant_result(
                    variant,
                    format!("{}-{}", config.study_id, variant.variant_id),
                    vec![ReasonCode::AblationVariantIgnored],
                    vec![AblationInterpretationFlag::UnknownOverrideIgnored],
                ));
                continue;
            };

            if baseline_failed && config.require_baseline_pass {
                push_unique(&mut reason_codes, ReasonCode::AblationNotComparable);
                push_unique(&mut flags, AblationInterpretationFlag::NotComparable);
                results.push(AblationVariantResult {
                    variant_id: variant.variant_id.clone(),
                    matrix_id: matrix.matrix_id,
                    dimension: variant.dimension,
                    status: AblationResultStatus::Skipped,
                    overrides: variant.overrides.clone(),
                    flags,
                    reason_codes,
                    delta: AblationDelta::default(),
                    avg_calibration_brier: None,
                    report: None,
                });
                continue;
            }

            let report = self.batch_runner.run_matrix(&matrix);
            let result = interpret_variant_result(
                variant,
                matrix.matrix_id,
                &baseline,
                report,
                reason_codes,
                flags,
            );
            results.push(result);
        }

        let sensitivity_summary = build_sensitivity_summary(&results);
        let next_step = select_next_step(&results, &sensitivity_summary);
        let mut reason_codes = config.reason_codes.clone();
        if baseline_failed {
            push_unique(&mut reason_codes, ReasonCode::AblationNotComparable);
        }
        let report = AblationStudyReport {
            study_id: config.study_id.clone(),
            baseline,
            variants: results,
            sensitivity_summary,
            next_step,
            warnings,
            reason_codes,
        };
        let _ = report.write_to_dir(&config.output_root);
        report
    }
}

fn prepare_variant_matrix(
    base_matrix: &ExperimentMatrixConfig,
    config: &AblationStudyConfig,
    variant: &AblationVariant,
) -> Option<(
    ExperimentMatrixConfig,
    Vec<ReasonCode>,
    Vec<AblationInterpretationFlag>,
)> {
    let mut matrix = base_matrix.clone();
    matrix.matrix_id = format!("{}-{}", config.study_id, variant.variant_id);
    let mut reason_codes = variant.reason_codes.clone();
    let mut flags = Vec::new();
    let mut applied_any = false;

    if variant.research_only {
        push_unique(&mut reason_codes, ReasonCode::ResearchOnlyOverride);
        push_unique(&mut flags, AblationInterpretationFlag::ResearchOnly);
    }

    for matrix_variant in &mut matrix.variants {
        match variant.dimension {
            AblationDimension::FeatureGroup => {
                let mut feature_config = matrix_variant
                    .overrides
                    .feature_config
                    .clone()
                    .unwrap_or_else(|| matrix.dataset_bundle.default_feature_config.clone());
                applied_any |= apply_feature_group_overrides(
                    &mut feature_config,
                    &variant.overrides,
                    variant.research_only,
                    &mut reason_codes,
                    &mut flags,
                );
                matrix_variant.overrides.feature_config = Some(feature_config);
            }
            AblationDimension::TripleBarrier => {
                let mut triple = matrix_variant
                    .overrides
                    .triple_barrier_config
                    .unwrap_or(matrix.dataset_bundle.default_triple_barrier_config);
                applied_any |= apply_triple_barrier_overrides(
                    &mut triple,
                    &variant.overrides,
                    &mut reason_codes,
                );
                matrix_variant.overrides.triple_barrier_config = Some(triple);
            }
            AblationDimension::CostModel => {
                let mut cost = matrix_variant
                    .overrides
                    .cost_model
                    .unwrap_or(matrix.dataset_bundle.default_cost_model);
                applied_any |=
                    apply_cost_model_overrides(&mut cost, &variant.overrides, &mut reason_codes);
                matrix_variant.overrides.cost_model = Some(cost);
            }
            AblationDimension::RiskGovernor => {
                let mut risk = matrix_variant
                    .overrides
                    .risk_profile
                    .unwrap_or(matrix.dataset_bundle.default_risk_config);
                applied_any |=
                    apply_risk_overrides(&mut risk, &variant.overrides, &mut reason_codes);
                matrix_variant.overrides.risk_profile = Some(risk);
            }
            AblationDimension::Chair => {
                let mut chair = matrix_variant
                    .overrides
                    .chair_config
                    .unwrap_or(matrix.dataset_bundle.default_chair_config);
                applied_any |=
                    apply_chair_overrides(&mut chair, &variant.overrides, &mut reason_codes);
                matrix_variant.overrides.chair_config = Some(chair);
            }
            AblationDimension::Regime => {
                let mut regime = matrix_variant
                    .overrides
                    .regime_config
                    .unwrap_or(matrix.dataset_bundle.default_regime_config);
                applied_any |=
                    apply_regime_overrides(&mut regime, &variant.overrides, &mut reason_codes);
                matrix_variant.overrides.regime_config = Some(regime);
            }
            AblationDimension::NoTradeScoring => {
                let mut scoring = matrix_variant
                    .overrides
                    .no_trade_score_config
                    .unwrap_or(matrix.dataset_bundle.default_no_trade_score_config);
                applied_any |=
                    apply_no_trade_overrides(&mut scoring, &variant.overrides, &mut reason_codes);
                matrix_variant.overrides.no_trade_score_config = Some(scoring);
            }
        }
    }

    if !applied_any {
        push_unique(&mut reason_codes, ReasonCode::AblationVariantIgnored);
        push_unique(
            &mut flags,
            AblationInterpretationFlag::UnknownOverrideIgnored,
        );
        return None;
    }

    Some((matrix, reason_codes, flags))
}

fn apply_feature_group_overrides(
    config: &mut crate::feature::FeatureConfig,
    overrides: &[AblationOverride],
    research_only: bool,
    reason_codes: &mut Vec<ReasonCode>,
    flags: &mut Vec<AblationInterpretationFlag>,
) -> bool {
    let mut applied = false;
    for override_item in overrides {
        match override_item.target.as_str() {
            "volume" => {
                if let Some(value) = override_item.value.as_bool() {
                    config.include_volume_features = value;
                    applied = true;
                }
            }
            "spread" | "spread_liquidity" => {
                if let Some(value) = override_item.value.as_bool() {
                    config.include_spread_features = value;
                    applied = true;
                }
            }
            "time" | "time_context" => {
                if let Some(value) = override_item.value.as_bool() {
                    config.include_time_features = value;
                    applied = true;
                }
            }
            "data_quality" => {
                if let Some(value) = override_item.value.as_bool() {
                    if !value && !research_only {
                        push_unique(reason_codes, ReasonCode::AblationVariantIgnored);
                        push_unique(flags, AblationInterpretationFlag::UnknownOverrideIgnored);
                    } else {
                        if !value {
                            push_unique(reason_codes, ReasonCode::ResearchOnlyOverride);
                            push_unique(flags, AblationInterpretationFlag::ResearchOnly);
                        }
                        config.include_data_quality_feature = value;
                        applied = true;
                    }
                }
            }
            _ => push_unique(reason_codes, ReasonCode::AblationVariantIgnored),
        }
    }
    applied
}

fn apply_triple_barrier_overrides(
    config: &mut crate::backtest::TripleBarrierConfig,
    overrides: &[AblationOverride],
    reason_codes: &mut Vec<ReasonCode>,
) -> bool {
    let mut applied = false;
    for override_item in overrides {
        match override_item.target.as_str() {
            "take_profit_pct" => {
                if let Some(value) = override_item.value.as_f64() {
                    config.take_profit_pct = value.max(0.0);
                    applied = true;
                }
            }
            "stop_loss_pct" => {
                if let Some(value) = override_item.value.as_f64() {
                    config.stop_loss_pct = value.max(0.0);
                    applied = true;
                }
            }
            "horizon_bars" => {
                if let Some(value) = override_item.value.as_usize() {
                    config.horizon_bars = value.max(1);
                    applied = true;
                }
            }
            _ => push_unique(reason_codes, ReasonCode::AblationVariantIgnored),
        }
    }
    applied
}

fn apply_cost_model_overrides(
    config: &mut crate::backtest::CostModel,
    overrides: &[AblationOverride],
    reason_codes: &mut Vec<ReasonCode>,
) -> bool {
    let mut applied = false;
    for override_item in overrides {
        match override_item.target.as_str() {
            "fee_bps" => {
                if let Some(value) = override_item.value.as_f64() {
                    config.fee_bps = value.max(0.0);
                    applied = true;
                }
            }
            "slippage_bps" => {
                if let Some(value) = override_item.value.as_f64() {
                    config.slippage_bps = value.max(0.0);
                    applied = true;
                }
            }
            "spread_bps" => {
                if let Some(value) = override_item.value.as_f64() {
                    config.spread_bps = Some(value.max(0.0));
                    applied = true;
                }
            }
            "min_cost_bps" => {
                if let Some(value) = override_item.value.as_f64() {
                    config.min_cost_bps = Some(value.max(0.0));
                    applied = true;
                }
            }
            _ => push_unique(reason_codes, ReasonCode::AblationVariantIgnored),
        }
    }
    applied
}

fn apply_risk_overrides(
    config: &mut crate::risk::GovernorConfig,
    overrides: &[AblationOverride],
    reason_codes: &mut Vec<ReasonCode>,
) -> bool {
    let mut applied = false;
    for override_item in overrides {
        match override_item.target.as_str() {
            "min_expected_edge" => {
                assign_f64(&mut config.min_expected_edge, override_item, &mut applied)
            }
            "min_confidence" => assign_f64(&mut config.min_confidence, override_item, &mut applied),
            "max_spread_bps" => assign_f64(&mut config.max_spread_bps, override_item, &mut applied),
            "min_data_quality" => {
                assign_f64(&mut config.min_data_quality, override_item, &mut applied)
            }
            "max_daily_loss_pct" => {
                assign_f64(&mut config.max_daily_loss_pct, override_item, &mut applied)
            }
            "max_allowed_volatility" => assign_f64(
                &mut config.max_allowed_volatility,
                override_item,
                &mut applied,
            ),
            "min_risk_reward" => {
                assign_f64(&mut config.min_risk_reward, override_item, &mut applied)
            }
            "max_total_exposure" => {
                assign_f64(&mut config.max_total_exposure, override_item, &mut applied)
            }
            "max_symbol_exposure" => {
                assign_f64(&mut config.max_symbol_exposure, override_item, &mut applied)
            }
            "min_trade_value" => {
                assign_f64(&mut config.min_trade_value, override_item, &mut applied)
            }
            _ => push_unique(reason_codes, ReasonCode::AblationVariantIgnored),
        }
    }
    applied
}

fn apply_chair_overrides(
    config: &mut crate::chair::ChairConfig,
    overrides: &[AblationOverride],
    reason_codes: &mut Vec<ReasonCode>,
) -> bool {
    let mut applied = false;
    for override_item in overrides {
        match override_item.target.as_str() {
            "strong_threshold" => {
                assign_f64(&mut config.strong_threshold, override_item, &mut applied)
            }
            "weak_threshold" => assign_f64(&mut config.weak_threshold, override_item, &mut applied),
            "allow_forced_contrarian" => assign_bool(
                &mut config.allow_forced_contrarian,
                override_item,
                &mut applied,
            ),
            "cluster_penalty_enabled" => assign_bool(
                &mut config.cluster_penalty_enabled,
                override_item,
                &mut applied,
            ),
            "groupthink_penalty_weight" => assign_f64(
                &mut config.groupthink_penalty_weight,
                override_item,
                &mut applied,
            ),
            "disagreement_penalty_weight" => assign_f64(
                &mut config.disagreement_penalty_weight,
                override_item,
                &mut applied,
            ),
            _ => push_unique(reason_codes, ReasonCode::AblationVariantIgnored),
        }
    }
    applied
}

fn apply_regime_overrides(
    config: &mut crate::regime::RegimeClassifierConfig,
    overrides: &[AblationOverride],
    reason_codes: &mut Vec<ReasonCode>,
) -> bool {
    let mut applied = false;
    for override_item in overrides {
        match override_item.target.as_str() {
            "min_data_quality" => {
                assign_f64(&mut config.min_data_quality, override_item, &mut applied)
            }
            "high_vol_threshold" => {
                assign_f64(&mut config.high_vol_threshold, override_item, &mut applied)
            }
            "panic_return_threshold" => assign_f64(
                &mut config.panic_return_threshold,
                override_item,
                &mut applied,
            ),
            "panic_volume_z_threshold" => assign_f64(
                &mut config.panic_volume_z_threshold,
                override_item,
                &mut applied,
            ),
            "risk_on_return_threshold" => assign_f64(
                &mut config.risk_on_return_threshold,
                override_item,
                &mut applied,
            ),
            "risk_off_return_threshold" => assign_f64(
                &mut config.risk_off_return_threshold,
                override_item,
                &mut applied,
            ),
            "range_return_abs_threshold" => assign_f64(
                &mut config.range_return_abs_threshold,
                override_item,
                &mut applied,
            ),
            _ => push_unique(reason_codes, ReasonCode::AblationVariantIgnored),
        }
    }
    applied
}

fn apply_no_trade_overrides(
    config: &mut crate::backtest::NoTradeScoreConfig,
    overrides: &[AblationOverride],
    reason_codes: &mut Vec<ReasonCode>,
) -> bool {
    let mut applied = false;
    for override_item in overrides {
        match override_item.target.as_str() {
            "avoided_loss_weight" => {
                assign_f64(&mut config.avoided_loss_weight, override_item, &mut applied)
            }
            "missed_gain_weight" => {
                assign_f64(&mut config.missed_gain_weight, override_item, &mut applied)
            }
            _ => push_unique(reason_codes, ReasonCode::AblationVariantIgnored),
        }
    }
    applied
}

fn interpret_variant_result(
    variant: &AblationVariant,
    matrix_id: String,
    baseline: &BaselineAblationSummary,
    report: BatchExperimentReport,
    mut reason_codes: Vec<ReasonCode>,
    mut flags: Vec<AblationInterpretationFlag>,
) -> AblationVariantResult {
    let avg_calibration_brier = average_calibration_brier(&report);
    let delta = AblationDelta {
        avg_net_return_pct: report.aggregate_benchmark.avg_net_return_pct
            - baseline.report.aggregate_benchmark.avg_net_return_pct,
        avg_max_drawdown_pct: report.aggregate_benchmark.avg_max_drawdown_pct
            - baseline.report.aggregate_benchmark.avg_max_drawdown_pct,
        avg_data_quality_score: report.aggregate_benchmark.avg_data_quality_score
            - baseline.report.aggregate_benchmark.avg_data_quality_score,
        avg_no_trade_rate: report.aggregate_benchmark.avg_no_trade_rate
            - baseline.report.aggregate_benchmark.avg_no_trade_rate,
        avg_denied_rate: report.aggregate_benchmark.avg_denied_rate
            - baseline.report.aggregate_benchmark.avg_denied_rate,
        avg_calibration_brier: match (avg_calibration_brier, baseline.avg_calibration_brier) {
            (Some(current), Some(previous)) => Some(current - previous),
            _ => None,
        },
    };

    if report.aggregate_benchmark.total_runs == 0 || report.aggregate_benchmark.failed_runs > 0 {
        push_unique(&mut reason_codes, ReasonCode::AblationNotComparable);
        push_unique(&mut flags, AblationInterpretationFlag::NotComparable);
    }
    if delta.avg_max_drawdown_pct > 0.01 {
        push_unique(&mut reason_codes, ReasonCode::AblationWorseDrawdown);
        push_unique(&mut flags, AblationInterpretationFlag::WorseDrawdown);
    }
    if delta.avg_calibration_brier.unwrap_or(0.0) > 0.02 {
        push_unique(&mut reason_codes, ReasonCode::AblationWorseCalibration);
        push_unique(&mut flags, AblationInterpretationFlag::WorseCalibration);
    }
    if matches!(
        variant.dimension,
        AblationDimension::CostModel | AblationDimension::NoTradeScoring
    ) && (delta.avg_net_return_pct < -0.005 || delta.avg_max_drawdown_pct > 0.005)
    {
        push_unique(&mut reason_codes, ReasonCode::AblationHighFragility);
        push_unique(&mut flags, AblationInterpretationFlag::HighFragility);
    }
    if !flags.contains(&AblationInterpretationFlag::NotComparable)
        && delta.avg_net_return_pct > 0.0
        && delta.avg_max_drawdown_pct <= 0.005
        && delta.avg_calibration_brier.unwrap_or(0.0) <= 0.02
        && delta.avg_data_quality_score >= -0.01
    {
        push_unique(&mut reason_codes, ReasonCode::AblationCandidateImprovement);
        push_unique(&mut flags, AblationInterpretationFlag::CandidateImprovement);
    }

    let status = if flags.contains(&AblationInterpretationFlag::NotComparable) {
        AblationResultStatus::Failed
    } else {
        AblationResultStatus::Applied
    };

    AblationVariantResult {
        variant_id: variant.variant_id.clone(),
        matrix_id,
        dimension: variant.dimension,
        status,
        overrides: variant.overrides.clone(),
        flags,
        reason_codes,
        delta,
        avg_calibration_brier,
        report: Some(report),
    }
}

fn skipped_variant_result(
    variant: &AblationVariant,
    matrix_id: String,
    reason_codes: Vec<ReasonCode>,
    flags: Vec<AblationInterpretationFlag>,
) -> AblationVariantResult {
    AblationVariantResult {
        variant_id: variant.variant_id.clone(),
        matrix_id,
        dimension: variant.dimension,
        status: AblationResultStatus::Skipped,
        overrides: variant.overrides.clone(),
        flags,
        reason_codes,
        delta: AblationDelta::default(),
        avg_calibration_brier: None,
        report: None,
    }
}

fn minimal_report(
    study_id: &str,
    warning: &str,
    reason_codes: Vec<ReasonCode>,
) -> AblationStudyReport {
    let baseline_report = empty_batch_report(&format!("{study_id}-baseline"));
    AblationStudyReport {
        study_id: study_id.to_string(),
        baseline: BaselineAblationSummary {
            matrix_id: format!("{study_id}-baseline"),
            avg_calibration_brier: None,
            report: baseline_report,
        },
        variants: Vec::new(),
        sensitivity_summary: SensitivitySummary::default(),
        next_step: NextStepRecommendation::NeedMoreExperiments,
        warnings: vec![warning.to_string()],
        reason_codes,
    }
}

fn empty_batch_report(matrix_id: &str) -> BatchExperimentReport {
    let aggregate_benchmark = build_aggregate_benchmark(&[]);
    let data_quality_summary = build_data_quality_aggregate(&[]);
    let risk_governor_summary = build_risk_governor_aggregate(&[], &[]);
    let model_comparison_summary = build_model_comparison_aggregate(&[]);
    let persona_readiness_summary = build_persona_readiness_summary(&[]);
    BatchExperimentReport {
        matrix_id: matrix_id.to_string(),
        run_summaries: Vec::new(),
        aggregate_benchmark,
        data_quality_summary,
        regime_summary: build_regime_aggregate(&[]),
        risk_governor_summary,
        model_comparison_summary,
        persona_readiness_summary: persona_readiness_summary.clone(),
        expansion_readiness: build_expansion_readiness_report(
            &[],
            &[],
            &build_data_quality_aggregate(&[]),
            &build_risk_governor_aggregate(&[], &[]),
            &build_model_comparison_aggregate(&[]),
            &persona_readiness_summary,
        ),
        reason_codes: vec![ReasonCode::DeterministicPath],
    }
}

fn average_calibration_brier(report: &BatchExperimentReport) -> Option<f64> {
    let values = report
        .run_summaries
        .iter()
        .filter_map(|summary| summary.calibration_brier)
        .collect::<Vec<_>>();
    if values.is_empty() {
        None
    } else {
        Some(values.iter().sum::<f64>() / values.len() as f64)
    }
}

fn assign_f64(target: &mut f64, override_item: &AblationOverride, applied: &mut bool) {
    if let Some(value) = override_item.value.as_f64() {
        *target = value;
        *applied = true;
    }
}

fn assign_bool(target: &mut bool, override_item: &AblationOverride, applied: &mut bool) {
    if let Some(value) = override_item.value.as_bool() {
        *target = value;
        *applied = true;
    }
}

fn push_unique<T: PartialEq>(values: &mut Vec<T>, value: T) {
    if !values.contains(&value) {
        values.push(value);
    }
}

fn default_output_root() -> String {
    "target/soma_ablations".to_string()
}

fn default_true() -> bool {
    true
}

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::{CoreReadinessStatus, ReasonCode};
use crate::data::ProviderKind;
use crate::eval::FeatureSchema;
use crate::experiment::{
    BenchmarkStorageAudit, CalibrationSummary, ModelComparisonSummary, ModelUsefulnessGateResult,
    OfficialAiBenchmarkReport, PerformanceSummary, RiskAiInteractionReport,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CoreCheckGateResult {
    pub core_check_ran: bool,
    #[serde(default)]
    pub core_status: Option<CoreReadinessStatus>,
    pub passed: bool,
    pub failed_reasons: Vec<String>,
    pub warnings: Vec<String>,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum OfficialDatasetCoverageStatus {
    CryptoOnly,
    MultiVenue,
    MissingEquityAuth,
    MissingOfficialData,
    InsufficientReadyEntries,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SelectedOfficialDatasets {
    pub selected_entries: Vec<String>,
    pub skipped_entries: Vec<String>,
    pub crypto_entries: Vec<String>,
    pub korean_equity_entries: Vec<String>,
    pub us_equity_entries: Vec<String>,
    pub missing_auth_entries: Vec<String>,
    pub failed_preflight_entries: Vec<String>,
    pub coverage_status: OfficialDatasetCoverageStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OfficialBenchmarkDatasetBundle {
    pub dataset_paths: Vec<String>,
    pub feature_schema: FeatureSchema,
    pub dataset_row_counts: BTreeMap<String, usize>,
    pub label_counts: BTreeMap<String, usize>,
    pub split_counts: BTreeMap<String, usize>,
    pub fold_counts: BTreeMap<String, usize>,
    pub no_lookahead_report: String,
    pub storage_bytes: usize,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ExternalTabularBenchmarkStage {
    pub training_requested: bool,
    pub training_ran: bool,
    #[serde(default)]
    pub training_backend_used: Option<String>,
    #[serde(default)]
    pub prediction_csv_path: Option<String>,
    #[serde(default)]
    pub model_card_path: Option<String>,
    pub prediction_validation_result: crate::model::PredictionValidationResult,
    pub schema_valid: bool,
    pub row_alignment_valid: bool,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CoreCheckedBenchmarkStatus {
    CoreBlocked,
    MissingOfficialData,
    MissingAuth,
    InsufficientOutcomes,
    BaselineOnlyEvaluated,
    ExternalModelEvaluated,
    ExternalTabularCandidate,
    PoorCalibration,
    PoorRiskBehavior,
    WorseThanBaseline,
    NeedMoreExperiments,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CoreCheckedBenchmarkRecommendation {
    MoreOfficialEvidence,
    ImproveDataFirst,
    ImproveSignalModelFirst,
    ImproveRiskGovernorFirst,
    BuildSequenceDatasetFirst,
    ExternalModelPrototype,
    HoldCurrentScope,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CoreCheckedBenchmarkReport {
    pub benchmark_id: String,
    pub core_check_gate: CoreCheckGateResult,
    #[serde(default)]
    pub dataset_selection: Option<SelectedOfficialDatasets>,
    #[serde(default)]
    pub dataset_bundle: Option<OfficialBenchmarkDatasetBundle>,
    #[serde(default)]
    pub baseline_report: Option<PerformanceSummary>,
    #[serde(default)]
    pub external_report: Option<PerformanceSummary>,
    #[serde(default)]
    pub calibration_report: Option<CalibrationSummary>,
    #[serde(default)]
    pub model_comparison_report: Option<ModelComparisonSummary>,
    #[serde(default)]
    pub risk_ai_interaction_report: Option<RiskAiInteractionReport>,
    pub storage_audit: BenchmarkStorageAudit,
    pub usefulness_gate_result: ModelUsefulnessGateResult,
    pub final_status: CoreCheckedBenchmarkStatus,
    pub next_recommendation: CoreCheckedBenchmarkRecommendation,
    pub blockers: Vec<String>,
    pub warnings: Vec<String>,
    pub reason_codes: Vec<ReasonCode>,
}

impl CoreCheckedBenchmarkReport {
    pub fn to_json_string(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|err| err.to_string())
    }

    pub fn to_text(&self) -> String {
        let mut lines = vec![
            format!("benchmark_id={}", self.benchmark_id),
            format!("core_check_passed={}", self.core_check_gate.passed),
            format!("final_status={:?}", self.final_status),
            format!("next_recommendation={:?}", self.next_recommendation),
            format!("blockers={}", self.blockers.join(" | ")),
            format!("warnings={}", self.warnings.join(" | ")),
            format!(
                "storage_budget_exceeded={}",
                self.storage_audit.budget_exceeded
            ),
        ];
        if let Some(selection) = &self.dataset_selection {
            lines.push(dataset_selection_to_text(selection));
        }
        if let Some(bundle) = &self.dataset_bundle {
            lines.push(format!("dataset_storage_bytes={}", bundle.storage_bytes));
            lines.push(format!("dataset_paths={}", bundle.dataset_paths.join("|")));
            lines.push(format!(
                "no_lookahead_report={}",
                bundle.no_lookahead_report
            ));
        }
        if let Some(stage) = &self.external_report {
            lines.push(format!("external_dataset_count={}", stage.dataset_count));
            lines.push(format!(
                "external_avg_net_return_pct={:.6}",
                stage.avg_net_return_pct
            ));
        }
        lines.join("\n")
    }

    pub fn to_markdown(&self) -> String {
        [
            format!("# {}", self.benchmark_id),
            format!("- final_status: `{:?}`", self.final_status),
            format!("- next_recommendation: `{:?}`", self.next_recommendation),
            format!("- core_check_passed: `{}`", self.core_check_gate.passed),
            format!(
                "- storage_budget_exceeded: `{}`",
                self.storage_audit.budget_exceeded
            ),
            format!("- blockers: {}", self.blockers.join(", ")),
            format!("- warnings: {}", self.warnings.join(", ")),
        ]
        .join("\n")
    }

    pub fn write_to_dir(&self, output_dir: &Path) -> Result<(), String> {
        fs::create_dir_all(output_dir).map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("core_checked_benchmark_report.json"),
            self.to_json_string()?,
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("core_checked_benchmark_report.txt"),
            self.to_text(),
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("core_checked_benchmark_report.md"),
            self.to_markdown(),
        )
        .map_err(|err| err.to_string())?;
        Ok(())
    }
}

pub fn dataset_selection_to_text(selection: &SelectedOfficialDatasets) -> String {
    [
        format!("selected_entries={}", selection.selected_entries.join("|")),
        format!("skipped_entries={}", selection.skipped_entries.join("|")),
        format!("crypto_entries={}", selection.crypto_entries.join("|")),
        format!(
            "korean_equity_entries={}",
            selection.korean_equity_entries.join("|")
        ),
        format!(
            "us_equity_entries={}",
            selection.us_equity_entries.join("|")
        ),
        format!(
            "missing_auth_entries={}",
            selection.missing_auth_entries.join("|")
        ),
        format!(
            "failed_preflight_entries={}",
            selection.failed_preflight_entries.join("|")
        ),
        format!("coverage_status={:?}", selection.coverage_status),
    ]
    .join("\n")
}

pub fn external_tabular_stage_to_text(stage: &ExternalTabularBenchmarkStage) -> String {
    [
        format!("training_requested={}", stage.training_requested),
        format!("training_ran={}", stage.training_ran),
        format!(
            "training_backend_used={}",
            stage.training_backend_used.clone().unwrap_or_default()
        ),
        format!(
            "prediction_csv_path={}",
            stage.prediction_csv_path.clone().unwrap_or_default()
        ),
        format!("schema_valid={}", stage.schema_valid),
        format!("row_alignment_valid={}", stage.row_alignment_valid),
        format!(
            "prediction_valid={}",
            stage.prediction_validation_result.valid
        ),
    ]
    .join("\n")
}

pub fn synthesize_risk_report(
    model_id: &str,
    summary: &crate::experiment::RiskGovernorSummary,
) -> RiskAiInteractionReport {
    RiskAiInteractionReport {
        model_id: model_id.to_string(),
        total_signals: summary.total_signals,
        approved_candidates: summary.total_signals.saturating_sub(summary.denied_by_risk),
        denied_by_risk: summary.denied_by_risk,
        no_trade_by_signal: summary.total_signals.saturating_sub(summary.denied_by_risk),
        no_trade_by_risk: summary.denied_by_risk,
        emergency_stop_count: summary.emergency_stop_count,
        cooldown_count: summary.cooldown_count,
        avoided_loss_count: 0,
        missed_gain_count: 0,
        defensive_value: summary.defensive_value,
        opportunity_cost: summary.opportunity_cost,
        denial_rate: summary.denial_rate,
        approval_rate: summary.approval_rate,
        reason_code_counts: vec![],
        warnings: if summary.stable {
            vec![]
        } else {
            vec!["risk governor stability fell below benchmark expectations".to_string()]
        },
        reason_codes: vec![ReasonCode::RiskAiInteractionBuilt],
    }
}

pub fn dataset_export_paths(report: &OfficialAiBenchmarkReport) -> Vec<PathBuf> {
    report
        .dataset_reports
        .iter()
        .filter_map(|dataset| dataset.dataset_export_dir.as_ref())
        .map(PathBuf::from)
        .collect()
}

pub fn build_dataset_bundle(
    dataset_paths: &[PathBuf],
    min_outcome_records: usize,
    max_allowed_storage_bytes: usize,
) -> Result<OfficialBenchmarkDatasetBundle, String> {
    let mut dataset_row_counts = BTreeMap::new();
    let mut label_counts = BTreeMap::new();
    let mut split_counts = BTreeMap::new();
    let mut fold_counts = BTreeMap::new();
    let mut all_feature_names = Vec::new();
    let mut total_storage = 0usize;
    let mut unsafe_rows = 0usize;
    let mut total_label_rows = 0usize;
    let mut reason_codes = vec![ReasonCode::DatasetExported];

    for dir in dataset_paths {
        let csv_path = dir.join("dataset.csv");
        if !csv_path.exists() {
            return Err(format!("missing dataset csv at {}", csv_path.display()));
        }
        let text = fs::read_to_string(&csv_path).map_err(|err| err.to_string())?;
        total_storage = total_storage.saturating_add(text.len());
        let mut lines = text.lines();
        let header = lines
            .next()
            .ok_or_else(|| format!("dataset csv missing header at {}", csv_path.display()))?;
        let headers = header
            .split(',')
            .map(|value| value.trim())
            .collect::<Vec<_>>();
        let feature_names = headers
            .iter()
            .filter_map(|name| parse_feature_name(name))
            .collect::<Vec<_>>();
        if all_feature_names.is_empty() {
            all_feature_names = feature_names.clone();
        }
        let split_index = headers.iter().position(|name| *name == "split_kind");
        let fold_index = headers.iter().position(|name| *name == "fold_id");
        let label_index = headers.iter().position(|name| *name == "label_outcome");
        for line in lines {
            let fields = line.split(',').collect::<Vec<_>>();
            *dataset_row_counts
                .entry(csv_path.display().to_string())
                .or_insert(0) += 1;
            if let Some(index) = split_index {
                let value = fields.get(index).copied().unwrap_or_default().to_string();
                *split_counts.entry(value.clone()).or_insert(0) += 1;
                if value == "Unsafe" {
                    unsafe_rows += 1;
                }
            }
            if let Some(index) = fold_index {
                let value = fields.get(index).copied().unwrap_or_default().to_string();
                *fold_counts.entry(value).or_insert(0) += 1;
            }
            if let Some(index) = label_index {
                let value = fields.get(index).copied().unwrap_or_default().to_string();
                if !value.is_empty() {
                    *label_counts.entry(value).or_insert(0) += 1;
                    total_label_rows += 1;
                }
            }
        }
    }

    if total_label_rows < min_outcome_records {
        reason_codes.push(ReasonCode::AiSignalInsufficientOutcomes);
    }
    if total_storage > max_allowed_storage_bytes {
        reason_codes.push(ReasonCode::BudgetExceeded);
    }

    Ok(OfficialBenchmarkDatasetBundle {
        dataset_paths: dataset_paths
            .iter()
            .map(|path| path.display().to_string())
            .collect(),
        feature_schema: FeatureSchema::from_feature_names(&all_feature_names),
        dataset_row_counts,
        label_counts,
        split_counts,
        fold_counts,
        no_lookahead_report: format!(
            "unsafe_rows={unsafe_rows};status={}",
            if unsafe_rows == 0 { "safe" } else { "unsafe" }
        ),
        storage_bytes: total_storage,
        reason_codes,
    })
}

pub fn is_non_official_provider(provider_kind: ProviderKind) -> bool {
    matches!(provider_kind, ProviderKind::MockFixture)
}

fn parse_feature_name(name: &str) -> Option<crate::feature::FeatureName> {
    use crate::feature::FeatureName;
    Some(match name {
        "close" => FeatureName::Close,
        "log_return_1" => FeatureName::LogReturn1,
        "log_return_3" => FeatureName::LogReturn3,
        "log_return_5" => FeatureName::LogReturn5,
        "log_return_10" => FeatureName::LogReturn10,
        "log_return_20" => FeatureName::LogReturn20,
        "close_position_in_range" => FeatureName::ClosePositionInRange,
        "high_low_range_pct" => FeatureName::HighLowRangePct,
        "candle_body_pct" => FeatureName::CandleBodyPct,
        "upper_wick_pct" => FeatureName::UpperWickPct,
        "lower_wick_pct" => FeatureName::LowerWickPct,
        "ma_5" => FeatureName::Ma5,
        "ma_20" => FeatureName::Ma20,
        "ma_5_over_ma_20" => FeatureName::Ma5OverMa20,
        "close_over_ma_20" => FeatureName::CloseOverMa20,
        "slope_ma_5" => FeatureName::SlopeMa5,
        "slope_ma_20" => FeatureName::SlopeMa20,
        "volume" => FeatureName::Volume,
        "volume_z_20" => FeatureName::VolumeZ20,
        "trade_value" => FeatureName::TradeValue,
        "trade_value_z_20" => FeatureName::TradeValueZ20,
        "volume_ratio_5_20" => FeatureName::VolumeRatio5_20,
        "atr_14" => FeatureName::Atr14,
        "realized_vol_10" => FeatureName::RealizedVol10,
        "realized_vol_20" => FeatureName::RealizedVol20,
        "bollinger_width_20" => FeatureName::BollingerWidth20,
        "range_volatility" => FeatureName::RangeVolatility,
        "vwap_20" => FeatureName::Vwap20,
        "close_over_vwap_20" => FeatureName::CloseOverVwap20,
        "spread_bps" => FeatureName::SpreadBps,
        "spread_bps_from_candle" => FeatureName::SpreadBpsFromCandle,
        "liquidity_score_heuristic" => FeatureName::LiquidityScoreHeuristic,
        "data_quality_score" => FeatureName::DataQualityScore,
        "minute_of_day_sin" => FeatureName::MinuteOfDaySin,
        "minute_of_day_cos" => FeatureName::MinuteOfDayCos,
        "day_of_week_sin" => FeatureName::DayOfWeekSin,
        "day_of_week_cos" => FeatureName::DayOfWeekCos,
        _ => return None,
    })
}

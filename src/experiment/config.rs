use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::backtest::{CostModel, NoTradeScoreConfig, Timeframe, TripleBarrierConfig};
use crate::chair::ChairConfig;
use crate::core::ReasonCode;
use crate::data::{CandleCsvConfig, CandleCsvFormat, DataValidationConfig};
use crate::eval::WalkForwardConfig;
use crate::feature::FeatureConfig;
use crate::regime::RegimeClassifierConfig;
use crate::risk::GovernorConfig;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExperimentMode {
    BaselineOnly,
    ExternalPredictionOnly,
    TrainAndCompare,
    DatasetExportOnly,
    ValidateDataOnly,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ExperimentConfig {
    pub experiment_id: String,
    pub symbol: String,
    pub data_path: String,
    pub csv_format: CandleCsvFormat,
    pub timeframe: Timeframe,
    pub resample_to: Option<Timeframe>,
    pub data_validation_config: DataValidationConfig,
    pub feature_config: FeatureConfig,
    pub regime_config: RegimeClassifierConfig,
    #[serde(default)]
    pub chair_config: ChairConfig,
    pub walk_forward_config: WalkForwardConfig,
    pub triple_barrier_config: TripleBarrierConfig,
    pub cost_model: CostModel,
    #[serde(default)]
    pub no_trade_score_config: NoTradeScoreConfig,
    pub risk_config: GovernorConfig,
    pub full_auto: bool,
    pub output_dir: String,
    pub run_python_training: bool,
    pub python_executable: Option<String>,
    pub training_script_path: Option<String>,
    pub prediction_csv_path: Option<String>,
    pub model_card_path: Option<String>,
    pub strict_schema_validation: bool,
    pub fail_on_bad_data: bool,
    pub created_at_ms: Option<u64>,
    pub mode: ExperimentMode,
    pub reason_codes: Vec<ReasonCode>,
}

impl ExperimentConfig {
    pub fn baseline_only(
        experiment_id: impl Into<String>,
        symbol: impl Into<String>,
        data_path: impl Into<String>,
        timeframe: Timeframe,
        output_dir: impl Into<String>,
    ) -> Self {
        Self {
            experiment_id: experiment_id.into(),
            symbol: symbol.into(),
            data_path: data_path.into(),
            csv_format: CandleCsvFormat::GenericOhlcv,
            timeframe,
            resample_to: None,
            data_validation_config: DataValidationConfig::default(),
            feature_config: FeatureConfig::default(),
            regime_config: RegimeClassifierConfig::default(),
            chair_config: ChairConfig::default(),
            walk_forward_config: WalkForwardConfig::default(),
            triple_barrier_config: TripleBarrierConfig {
                take_profit_pct: 0.02,
                stop_loss_pct: 0.01,
                horizon_bars: 8,
                fee_bps: 2.0,
                slippage_bps: 2.0,
                side: crate::core::Side::Long,
                use_high_low_intrabar: true,
            },
            cost_model: CostModel {
                fee_bps: 2.0,
                slippage_bps: 2.0,
                spread_bps: Some(2.0),
                min_cost_bps: None,
            },
            no_trade_score_config: NoTradeScoreConfig::default(),
            risk_config: GovernorConfig::default(),
            full_auto: false,
            output_dir: output_dir.into(),
            run_python_training: false,
            python_executable: None,
            training_script_path: None,
            prediction_csv_path: None,
            model_card_path: None,
            strict_schema_validation: true,
            fail_on_bad_data: true,
            created_at_ms: None,
            mode: ExperimentMode::BaselineOnly,
            reason_codes: vec![ReasonCode::DeterministicPath],
        }
    }

    pub fn dataset_export_only(
        experiment_id: impl Into<String>,
        symbol: impl Into<String>,
        data_path: impl Into<String>,
        timeframe: Timeframe,
        output_dir: impl Into<String>,
    ) -> Self {
        let mut config =
            Self::baseline_only(experiment_id, symbol, data_path, timeframe, output_dir);
        config.mode = ExperimentMode::DatasetExportOnly;
        config
    }

    pub fn validate_local_paths(&self) -> Vec<ReasonCode> {
        let paths = [
            Some(self.data_path.as_str()),
            Some(self.output_dir.as_str()),
            self.training_script_path.as_deref(),
            self.prediction_csv_path.as_deref(),
            self.model_card_path.as_deref(),
        ];
        if paths.into_iter().flatten().any(is_remote_like) {
            vec![
                ReasonCode::LocalPathRejected,
                ReasonCode::ExperimentConfigInvalid,
            ]
        } else {
            Vec::new()
        }
    }

    pub fn output_bundle_dir(&self) -> PathBuf {
        PathBuf::from(&self.output_dir).join(&self.experiment_id)
    }

    pub fn output_paths(&self) -> BTreeMap<String, String> {
        let dir = self.output_bundle_dir();
        BTreeMap::from([
            (
                "manifest".to_string(),
                dir.join("manifest.txt").display().to_string(),
            ),
            (
                "data_quality_report".to_string(),
                dir.join("data_quality_report.txt").display().to_string(),
            ),
            (
                "dataset".to_string(),
                dir.join("dataset.csv").display().to_string(),
            ),
            (
                "baseline_report".to_string(),
                dir.join("baseline_report.txt").display().to_string(),
            ),
            (
                "predictions".to_string(),
                dir.join("predictions.csv").display().to_string(),
            ),
            (
                "model_card".to_string(),
                dir.join("model_card.md").display().to_string(),
            ),
            (
                "external_report".to_string(),
                dir.join("external_report.txt").display().to_string(),
            ),
            (
                "comparison_report".to_string(),
                dir.join("comparison_report.txt").display().to_string(),
            ),
            (
                "experiment_summary".to_string(),
                dir.join("experiment_summary.txt").display().to_string(),
            ),
        ])
    }

    pub fn build_csv_config(&self) -> CandleCsvConfig {
        CandleCsvConfig {
            format: self.csv_format.clone(),
            symbol: self.symbol.clone(),
            timeframe: self.timeframe,
            strict: self.data_validation_config.strict,
            allow_repair_sort: self.data_validation_config.allow_sort_repair,
            allow_drop_invalid_rows: !self.fail_on_bad_data,
            max_invalid_rows: if self.fail_on_bad_data {
                0
            } else {
                usize::MAX / 4
            },
            ..CandleCsvConfig::default()
        }
    }

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
}

fn is_remote_like(path: &str) -> bool {
    path.contains("://")
}

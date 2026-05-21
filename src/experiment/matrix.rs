use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::backtest::{CostModel, NoTradeScoreConfig, Timeframe, TripleBarrierConfig};
use crate::chair::ChairConfig;
use crate::core::ReasonCode;
use crate::data::{AssetClass, CandleCsvFormat, DataProvenance, DataValidationConfig, MarketVenue};
use crate::eval::WalkForwardConfig;
use crate::feature::FeatureConfig;
use crate::regime::RegimeClassifierConfig;
use crate::risk::GovernorConfig;

use super::config::{ExperimentConfig, ExperimentMode};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DatasetEntry {
    pub dataset_id: String,
    pub symbol: String,
    pub data_path: String,
    pub csv_format: CandleCsvFormat,
    pub timeframe: Timeframe,
    pub resample_to: Option<Timeframe>,
    pub venue: MarketVenue,
    pub asset_class: AssetClass,
    #[serde(default)]
    pub provenance: Option<DataProvenance>,
    pub enabled: bool,
    pub tags: Vec<String>,
    pub expected_min_rows: Option<usize>,
    pub notes: Option<String>,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DatasetBundleConfig {
    pub bundle_id: String,
    pub entries: Vec<DatasetEntry>,
    pub default_data_validation_config: DataValidationConfig,
    pub default_feature_config: FeatureConfig,
    #[serde(default)]
    pub default_regime_config: RegimeClassifierConfig,
    #[serde(default)]
    pub default_chair_config: ChairConfig,
    pub default_walk_forward_config: WalkForwardConfig,
    pub default_triple_barrier_config: TripleBarrierConfig,
    pub default_cost_model: CostModel,
    #[serde(default)]
    pub default_no_trade_score_config: NoTradeScoreConfig,
    pub default_risk_config: GovernorConfig,
    pub output_root: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ExperimentVariantOverrides {
    pub timeframe: Option<Timeframe>,
    pub resample_to: Option<Timeframe>,
    pub triple_barrier_config: Option<TripleBarrierConfig>,
    pub cost_model: Option<CostModel>,
    pub risk_profile: Option<GovernorConfig>,
    pub feature_config: Option<FeatureConfig>,
    pub regime_config: Option<RegimeClassifierConfig>,
    pub chair_config: Option<ChairConfig>,
    pub walk_forward_config: Option<WalkForwardConfig>,
    pub no_trade_score_config: Option<NoTradeScoreConfig>,
    pub run_python_training: Option<bool>,
    pub python_executable: Option<String>,
    pub training_script_path: Option<String>,
    pub prediction_csv_path: Option<String>,
    pub model_card_path: Option<String>,
    pub strict_schema_validation: Option<bool>,
    pub fail_on_bad_data: Option<bool>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ExperimentVariant {
    pub variant_id: String,
    pub mode: ExperimentMode,
    pub overrides: ExperimentVariantOverrides,
    pub enabled: bool,
    pub tags: Vec<String>,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ExperimentMatrixConfig {
    pub matrix_id: String,
    pub dataset_bundle: DatasetBundleConfig,
    pub variants: Vec<ExperimentVariant>,
    pub continue_on_failure: bool,
    pub require_all_pass: bool,
    pub deterministic_run_id: Option<String>,
    pub reason_codes: Vec<ReasonCode>,
}

impl ExperimentMatrixConfig {
    pub fn from_toml_path(path: &Path) -> Result<Self, String> {
        let contents = fs::read_to_string(path).map_err(|err| err.to_string())?;
        toml::from_str(&contents).map_err(|err| err.to_string())
    }

    pub fn to_toml_string(&self) -> Result<String, String> {
        toml::to_string_pretty(self).map_err(|err| err.to_string())
    }

    pub fn validate_local_paths(&self) -> Vec<ReasonCode> {
        let mut reasons = Vec::new();
        if self.dataset_bundle.output_root.contains("://") {
            reasons.push(ReasonCode::LocalPathRejected);
        }
        if self
            .dataset_bundle
            .entries
            .iter()
            .any(|entry| entry.data_path.contains("://"))
        {
            reasons.push(ReasonCode::LocalPathRejected);
        }
        if reasons.is_empty() {
            reasons
        } else {
            vec![
                ReasonCode::LocalPathRejected,
                ReasonCode::ExperimentConfigInvalid,
            ]
        }
    }

    pub fn build_experiment_config(
        &self,
        dataset: &DatasetEntry,
        variant: &ExperimentVariant,
    ) -> ExperimentConfig {
        let timeframe = variant.overrides.timeframe.unwrap_or(dataset.timeframe);
        ExperimentConfig {
            experiment_id: if let Some(run_id) = &self.deterministic_run_id {
                format!("{run_id}-{}-{}", dataset.dataset_id, variant.variant_id)
            } else {
                format!(
                    "{}-{}-{}",
                    self.matrix_id, dataset.dataset_id, variant.variant_id
                )
            },
            symbol: dataset.symbol.clone(),
            data_path: dataset.data_path.clone(),
            csv_format: dataset.csv_format.clone(),
            timeframe,
            resample_to: variant.overrides.resample_to.or(dataset.resample_to),
            data_validation_config: self.dataset_bundle.default_data_validation_config.clone(),
            feature_config: variant
                .overrides
                .feature_config
                .clone()
                .unwrap_or_else(|| self.dataset_bundle.default_feature_config.clone()),
            regime_config: variant
                .overrides
                .regime_config
                .unwrap_or(self.dataset_bundle.default_regime_config),
            chair_config: variant
                .overrides
                .chair_config
                .unwrap_or(self.dataset_bundle.default_chair_config),
            walk_forward_config: variant
                .overrides
                .walk_forward_config
                .unwrap_or(self.dataset_bundle.default_walk_forward_config),
            triple_barrier_config: variant
                .overrides
                .triple_barrier_config
                .unwrap_or(self.dataset_bundle.default_triple_barrier_config),
            cost_model: variant
                .overrides
                .cost_model
                .unwrap_or(self.dataset_bundle.default_cost_model),
            no_trade_score_config: variant
                .overrides
                .no_trade_score_config
                .unwrap_or(self.dataset_bundle.default_no_trade_score_config),
            risk_config: variant
                .overrides
                .risk_profile
                .unwrap_or(self.dataset_bundle.default_risk_config),
            full_auto: false,
            output_dir: self.dataset_bundle.output_root.clone(),
            run_python_training: variant
                .overrides
                .run_python_training
                .unwrap_or(matches!(variant.mode, ExperimentMode::TrainAndCompare)),
            python_executable: variant.overrides.python_executable.clone(),
            training_script_path: variant.overrides.training_script_path.clone(),
            prediction_csv_path: variant.overrides.prediction_csv_path.clone(),
            model_card_path: variant.overrides.model_card_path.clone(),
            strict_schema_validation: variant.overrides.strict_schema_validation.unwrap_or(true),
            fail_on_bad_data: variant.overrides.fail_on_bad_data.unwrap_or(true),
            created_at_ms: self
                .deterministic_run_id
                .as_ref()
                .map(|seed| crate::core::stable_hash(seed)),
            mode: variant.mode,
            reason_codes: vec![ReasonCode::DeterministicPath],
        }
    }
}

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::core::ReasonCode;
use crate::data::DataManifest;
use crate::eval::FeatureSchema;

use super::config::{ExperimentConfig, ExperimentMode};
use super::stage::{ExperimentStage, StageStatus};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ExperimentManifest {
    pub manifest_version: u32,
    pub experiment_id: String,
    pub input_data_manifest: DataManifest,
    pub feature_schema: FeatureSchema,
    pub feature_config_summary: String,
    pub walk_forward_config_summary: String,
    pub label_config_summary: String,
    pub cost_model_summary: String,
    pub regime_config_summary: String,
    pub chair_config_summary: String,
    pub no_trade_config_summary: String,
    pub risk_config_summary: String,
    pub mode: ExperimentMode,
    pub output_paths: BTreeMap<String, String>,
    pub stage_statuses: BTreeMap<ExperimentStage, StageStatus>,
    pub reason_codes: Vec<ReasonCode>,
}

impl ExperimentManifest {
    pub fn new(
        config: &ExperimentConfig,
        input_data_manifest: DataManifest,
        feature_schema: FeatureSchema,
    ) -> Self {
        let stage_statuses = ExperimentStage::all()
            .into_iter()
            .map(|stage| (stage, StageStatus::Pending))
            .collect::<BTreeMap<_, _>>();
        Self {
            manifest_version: 1,
            experiment_id: config.experiment_id.clone(),
            input_data_manifest,
            feature_schema,
            feature_config_summary: format!(
                "include_volume_features={},include_spread_features={},include_data_quality_feature={},include_time_features={},min_required_bars={}",
                config.feature_config.include_volume_features,
                config.feature_config.include_spread_features,
                config.feature_config.include_data_quality_feature,
                config.feature_config.include_time_features,
                config.feature_config.min_required_bars,
            ),
            walk_forward_config_summary: format!(
                "train_window_bars={},validation_window_bars={:?},test_window_bars={},step_bars={},embargo_bars={},min_train_bars={},max_folds={:?},allow_partial_last_fold={}",
                config.walk_forward_config.train_window_bars,
                config.walk_forward_config.validation_window_bars,
                config.walk_forward_config.test_window_bars,
                config.walk_forward_config.step_bars,
                config.walk_forward_config.embargo_bars,
                config.walk_forward_config.min_train_bars,
                config.walk_forward_config.max_folds,
                config.walk_forward_config.allow_partial_last_fold,
            ),
            label_config_summary: format!(
                "take_profit_pct={:.6},stop_loss_pct={:.6},horizon_bars={},fee_bps={:.4},slippage_bps={:.4},side={:?}",
                config.triple_barrier_config.take_profit_pct,
                config.triple_barrier_config.stop_loss_pct,
                config.triple_barrier_config.horizon_bars,
                config.triple_barrier_config.fee_bps,
                config.triple_barrier_config.slippage_bps,
                config.triple_barrier_config.side,
            ),
            cost_model_summary: format!(
                "fee_bps={:.4},slippage_bps={:.4},spread_bps={:?}",
                config.cost_model.fee_bps,
                config.cost_model.slippage_bps,
                config.cost_model.spread_bps,
            ),
            regime_config_summary: format!(
                "min_data_quality={:.4},high_vol_threshold={:.4},panic_return_threshold={:.4},panic_volume_z_threshold={:.4},risk_on_return_threshold={:.4},risk_off_return_threshold={:.4},range_return_abs_threshold={:.4}",
                config.regime_config.min_data_quality,
                config.regime_config.high_vol_threshold,
                config.regime_config.panic_return_threshold,
                config.regime_config.panic_volume_z_threshold,
                config.regime_config.risk_on_return_threshold,
                config.regime_config.risk_off_return_threshold,
                config.regime_config.range_return_abs_threshold,
            ),
            chair_config_summary: format!(
                "strong_threshold={:.4},weak_threshold={:.4},allow_forced_contrarian={},cluster_penalty_enabled={},groupthink_penalty_weight={:.4},disagreement_penalty_weight={:.4}",
                config.chair_config.strong_threshold,
                config.chair_config.weak_threshold,
                config.chair_config.allow_forced_contrarian,
                config.chair_config.cluster_penalty_enabled,
                config.chair_config.groupthink_penalty_weight,
                config.chair_config.disagreement_penalty_weight,
            ),
            no_trade_config_summary: format!(
                "avoided_loss_weight={:.4},missed_gain_weight={:.4}",
                config.no_trade_score_config.avoided_loss_weight,
                config.no_trade_score_config.missed_gain_weight,
            ),
            risk_config_summary: format!(
                "min_expected_edge={:.6},min_confidence={:.4},max_spread_bps={:.4},min_data_quality={:.4},max_daily_loss_pct={:.4}",
                config.risk_config.min_expected_edge,
                config.risk_config.min_confidence,
                config.risk_config.max_spread_bps,
                config.risk_config.min_data_quality,
                config.risk_config.max_daily_loss_pct,
            ),
            mode: config.mode,
            output_paths: config.output_paths(),
            stage_statuses,
            reason_codes: vec![ReasonCode::ExperimentManifestBuilt],
        }
    }

    pub fn set_stage_status(&mut self, stage: ExperimentStage, status: StageStatus) {
        self.stage_statuses.insert(stage, status);
    }

    pub fn mark_remaining_skipped(&mut self) {
        for stage in ExperimentStage::all() {
            if self.stage_statuses.get(&stage) == Some(&StageStatus::Pending) {
                self.stage_statuses.insert(stage, StageStatus::Skipped);
            }
        }
    }

    pub fn to_deterministic_string(&self) -> String {
        let mut lines = vec![
            format!("manifest_version={}", self.manifest_version),
            format!("experiment_id={}", self.experiment_id),
            format!("mode={:?}", self.mode),
            format!("input_dataset_id={}", self.input_data_manifest.dataset_id),
            format!(
                "feature_schema_version={}",
                self.feature_schema.schema_version
            ),
            format!("feature_schema_hash={}", self.feature_schema.checksum),
            format!("feature_config={}", self.feature_config_summary),
            format!("walk_forward_config={}", self.walk_forward_config_summary),
            format!("label_config={}", self.label_config_summary),
            format!("cost_model={}", self.cost_model_summary),
            format!("regime_config={}", self.regime_config_summary),
            format!("chair_config={}", self.chair_config_summary),
            format!("no_trade_config={}", self.no_trade_config_summary),
            format!("risk_config={}", self.risk_config_summary),
        ];
        for stage in ExperimentStage::all() {
            let status = self
                .stage_statuses
                .get(&stage)
                .copied()
                .unwrap_or(StageStatus::Pending);
            lines.push(format!("stage.{}={:?}", stage.as_str(), status));
        }
        for (key, value) in &self.output_paths {
            lines.push(format!("output.{}={}", key, value));
        }
        lines.push(format!(
            "reason_codes={}",
            self.reason_codes
                .iter()
                .map(|reason| format!("{reason:?}"))
                .collect::<Vec<_>>()
                .join("|")
        ));
        lines.join("\n")
    }
}

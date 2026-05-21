use serde::{Deserialize, Serialize};

use crate::core::ReasonCode;
use crate::eval::{
    DatasetExportConfig, DatasetOutputFormat, DatasetSplitKind, WalkForwardEvaluator,
    WalkForwardSplit,
};
use crate::feature::FeatureConfig;

use super::{
    DataQualityReport, DataQualitySeverity, EvidenceSourceKind, LoadedCandleData,
    LocalDataOnboardingConfig,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EvidenceTargetEstimate {
    pub estimated_usable_datasets: usize,
    pub estimated_walk_forward_folds: usize,
    pub estimated_outcome_records: usize,
    pub estimated_comparable_variants: usize,
    pub enough_for_minimum_real_evidence: bool,
    pub missing_usable_datasets: usize,
    pub missing_outcome_records: usize,
    pub missing_comparable_variants: usize,
    pub assumptions: Vec<String>,
    pub reason_codes: Vec<ReasonCode>,
}

pub fn estimate_evidence_targets(
    config: &LocalDataOnboardingConfig,
    loaded: &LoadedCandleData,
) -> EvidenceTargetEstimate {
    let walk_forward = config.resolved_walk_forward_config();
    let split = WalkForwardSplit::generate(&loaded.series, walk_forward);
    let folds = split.folds.len();
    let quality = &loaded.quality_report;
    let triple_barrier = config.resolved_triple_barrier_config();
    let feature_config = FeatureConfig::default();

    let row_floor = config
        .min_rows_for_preflight
        .max(feature_config.min_required_bars)
        .max(triple_barrier.horizon_bars.saturating_add(1));
    let source_kind = config.source_kind.unwrap_or(EvidenceSourceKind::RealLocal);
    let real_local_eligible = source_kind.readiness_eligible()
        && (config.user_supplied || source_kind == EvidenceSourceKind::OfficialApiCollected);

    let estimated_outcome_records = if !real_local_eligible
        || matches!(
            quality.severity,
            DataQualitySeverity::Bad | DataQualitySeverity::Unusable
        )
        || loaded.series.len() < row_floor
        || folds == 0
    {
        0
    } else {
        conservative_outcome_estimate(loaded, walk_forward, triple_barrier.horizon_bars)
    };
    let estimated_comparable_variants = conservative_variant_estimate(
        estimated_outcome_records,
        config.target_min_outcomes,
        config.target_min_comparable_variants,
    );
    let estimated_usable_datasets = usize::from(
        real_local_eligible
            && !matches!(
                quality.severity,
                DataQualitySeverity::Bad | DataQualitySeverity::Unusable
            )
            && loaded.series.len() >= row_floor
            && folds > 0
            && estimated_outcome_records > 0,
    );
    let missing_usable_datasets = config
        .target_min_usable_datasets
        .saturating_sub(estimated_usable_datasets);
    let missing_outcome_records = config
        .target_min_outcomes
        .saturating_sub(estimated_outcome_records);
    let missing_comparable_variants = config
        .target_min_comparable_variants
        .saturating_sub(estimated_comparable_variants);
    EvidenceTargetEstimate {
        estimated_usable_datasets,
        estimated_walk_forward_folds: folds,
        estimated_outcome_records,
        estimated_comparable_variants,
        enough_for_minimum_real_evidence: missing_usable_datasets == 0
            && missing_outcome_records == 0
            && missing_comparable_variants == 0,
        missing_usable_datasets,
        missing_outcome_records,
        missing_comparable_variants,
        assumptions: build_assumptions(config, quality, folds),
        reason_codes: dedupe_reasons({
            let mut reasons = quality.reason_codes.clone();
            reasons.extend(split.reason_codes);
            reasons.push(ReasonCode::EvidenceEstimateBuilt);
            reasons
        }),
    }
}

fn conservative_outcome_estimate(
    loaded: &LoadedCandleData,
    walk_forward: crate::eval::WalkForwardConfig,
    horizon_bars: usize,
) -> usize {
    let evaluator = WalkForwardEvaluator {
        triple_barrier_config: crate::backtest::TripleBarrierConfig {
            take_profit_pct: 0.02,
            stop_loss_pct: 0.01,
            horizon_bars,
            fee_bps: 2.0,
            slippage_bps: 2.0,
            side: crate::core::Side::Long,
            use_high_low_intrabar: true,
        },
        ..WalkForwardEvaluator::default()
    };
    let split = evaluator.split(&loaded.series, walk_forward);
    let dataset = evaluator.export_dataset(
        &loaded.series,
        &split,
        &DatasetExportConfig {
            include_labels: true,
            include_metadata: true,
            include_reason_codes: true,
            output_format: DatasetOutputFormat::Csv,
        },
    );
    let labeled_eval_rows = dataset
        .rows
        .iter()
        .filter(|row| {
            matches!(
                row.split_kind,
                DatasetSplitKind::Validation | DatasetSplitKind::Test
            ) && row.label_outcome.is_some()
        })
        .count();
    let horizon_floor = horizon_bars.max(1);
    labeled_eval_rows.min(loaded.series.len() / horizon_floor)
}

fn conservative_variant_estimate(
    estimated_outcome_records: usize,
    target_min_outcomes: usize,
    target_min_comparable_variants: usize,
) -> usize {
    if estimated_outcome_records >= target_min_outcomes {
        target_min_comparable_variants
    } else if estimated_outcome_records >= target_min_outcomes.saturating_div(2).max(1) {
        1.min(target_min_comparable_variants)
    } else {
        0
    }
}

fn build_assumptions(
    config: &LocalDataOnboardingConfig,
    quality: &DataQualityReport,
    folds: usize,
) -> Vec<String> {
    let mut assumptions = vec![
        "conservative estimate only".to_string(),
        "same-input local-only preflight is deterministic".to_string(),
        "synthetic/test evidence is excluded from readiness".to_string(),
    ];
    let source_kind = config.source_kind.unwrap_or(EvidenceSourceKind::RealLocal);
    if !config.user_supplied && source_kind != EvidenceSourceKind::OfficialApiCollected {
        assumptions.push("user_supplied=false prevents real-local readiness".to_string());
    }
    if source_kind == EvidenceSourceKind::OfficialApiCollected {
        assumptions
            .push("official-api-collected local copies can count after validation".to_string());
    }
    if matches!(
        quality.severity,
        DataQualitySeverity::Bad | DataQualitySeverity::Unusable
    ) {
        assumptions.push("bad or unusable data yields zero readiness estimate".to_string());
    }
    if folds == 0 {
        assumptions.push("no walk-forward folds means zero outcome estimate".to_string());
    }
    assumptions
}

fn dedupe_reasons(values: Vec<ReasonCode>) -> Vec<ReasonCode> {
    let mut deduped = Vec::new();
    for value in values {
        if !deduped.contains(&value) {
            deduped.push(value);
        }
    }
    deduped
}

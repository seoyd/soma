pub mod dataset;
pub mod feature_schema;
pub mod leakage;
pub mod metrics;
pub mod report;
pub mod walk_forward;

pub use dataset::{
    DatasetExportConfig, DatasetFrame, DatasetOutputFormat, DatasetRow, DatasetSplitKind,
    dataset_row_id,
};
pub use feature_schema::FeatureSchema;
pub use leakage::{LeakageGuard, LeakageReport};
pub use metrics::{
    CalibrationBin, CalibrationMetrics, ChairMetrics, DecisionMetrics, NoTradeMetrics,
    PersonaFoldMetrics, RegimeMetrics, RiskGovernorMetrics, TradeMetrics,
    compute_calibration_metrics, compute_chair_metrics, compute_decision_metrics,
    compute_no_trade_metrics, compute_persona_metrics, compute_regime_metrics,
    compute_risk_metrics, compute_trade_metrics,
};
pub use report::{FoldReport, WalkForwardAggregateMetrics, WalkForwardReport};
pub use walk_forward::{
    WalkForwardConfig, WalkForwardEvaluator, WalkForwardFold, WalkForwardSplit,
};

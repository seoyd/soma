pub mod engine;
pub mod quality;
pub mod rolling;
pub mod types;

pub use engine::FeatureEngine;
pub use quality::{DataQualityResult, assess_data_quality};
pub use rolling::{
    atr, clamp_finite, log_return, pct_change, realized_volatility, rolling_max, rolling_mean,
    rolling_min, rolling_std, rolling_sum, rolling_zscore, safe_div, true_range,
};
pub use types::{FeatureConfig, FeatureFrame, FeatureName, FeatureValue, FeatureVector};

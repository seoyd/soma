use soma_zero::{FeatureConfig, FeatureEngine, FeatureName};

#[test]
fn feature_group_toggles_change_schema_deterministically() {
    let default_names = FeatureEngine::default().feature_names();
    assert!(default_names.contains(&FeatureName::Volume));
    assert!(default_names.contains(&FeatureName::SpreadBps));
    assert!(default_names.contains(&FeatureName::DataQualityScore));

    let without_volume = FeatureEngine {
        config: FeatureConfig {
            include_volume_features: false,
            ..FeatureConfig::default()
        },
    }
    .feature_names();
    assert!(!without_volume.contains(&FeatureName::Volume));
    assert!(!without_volume.contains(&FeatureName::TradeValue));

    let without_spread = FeatureEngine {
        config: FeatureConfig {
            include_spread_features: false,
            ..FeatureConfig::default()
        },
    }
    .feature_names();
    assert!(!without_spread.contains(&FeatureName::SpreadBps));
    assert!(!without_spread.contains(&FeatureName::LiquidityScoreHeuristic));

    let without_quality = FeatureEngine {
        config: FeatureConfig {
            include_data_quality_feature: false,
            ..FeatureConfig::default()
        },
    }
    .feature_names();
    assert!(!without_quality.contains(&FeatureName::DataQualityScore));
}

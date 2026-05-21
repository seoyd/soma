use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_hash};
use crate::feature::{FeatureEngine, FeatureName};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FeatureSchema {
    pub schema_version: u32,
    pub feature_names: Vec<FeatureName>,
    pub feature_count: usize,
    pub checksum: u64,
    pub created_by: String,
}

impl FeatureSchema {
    pub fn from_engine(engine: &FeatureEngine) -> Self {
        Self::from_feature_names(&engine.feature_names())
    }

    pub fn from_feature_names(feature_names: &[FeatureName]) -> Self {
        let checksum_input = feature_names
            .iter()
            .map(|feature| feature.as_str())
            .collect::<Vec<_>>()
            .join("|");
        Self {
            schema_version: 1,
            feature_names: feature_names.to_vec(),
            feature_count: feature_names.len(),
            checksum: stable_hash(&checksum_input),
            created_by: "feature_engine_v0".to_string(),
        }
    }

    pub fn validate_feature_names(
        &self,
        feature_names: &[FeatureName],
    ) -> Result<(), Vec<ReasonCode>> {
        let current = Self::from_feature_names(feature_names);
        if self.feature_names == current.feature_names
            && self.feature_count == current.feature_count
            && self.checksum == current.checksum
        {
            Ok(())
        } else {
            Err(vec![ReasonCode::FeatureSchemaMismatch])
        }
    }
}

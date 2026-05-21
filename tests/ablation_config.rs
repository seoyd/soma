mod common;

use soma_zero::{
    AblationDimension, AblationInterpretationFlag, AblationResultStatus, AblationRunner,
    AblationStudyConfig, AblationValue, AblationVariant, ReasonCode,
};

use crate::common::{baseline_variant, batch_matrix, dataset_entry, output_dir};

#[test]
fn ablation_config_rejects_remote_like_paths() {
    let config = AblationStudyConfig {
        study_id: "remote".to_string(),
        output_root: "https://example.invalid/out".to_string(),
        ..AblationStudyConfig::default()
    };
    let reasons = config.validate_local_paths();
    assert!(reasons.contains(&ReasonCode::LocalPathRejected));
    assert!(reasons.contains(&ReasonCode::ExperimentConfigInvalid));
}

#[test]
fn unknown_override_is_skipped_with_reason_code() {
    let base = batch_matrix(
        "ablation-unknown-base",
        vec![dataset_entry("valid", "generic_ohlcv_valid.csv", true)],
        vec![baseline_variant("baseline_5m", true)],
    );
    let report = AblationRunner::default().run_study(&AblationStudyConfig {
        study_id: "ablation-unknown".to_string(),
        embedded_base_matrix: Some(base),
        output_root: output_dir("ablation-unknown").display().to_string(),
        variants: vec![AblationVariant {
            variant_id: "unknown-target".to_string(),
            dimension: AblationDimension::FeatureGroup,
            overrides: vec![soma_zero::AblationOverride {
                target: "mystery_group".to_string(),
                value: AblationValue::Bool(false),
            }],
            research_only: false,
            enabled: true,
            tags: vec![],
            notes: None,
            reason_codes: vec![],
        }],
        ..AblationStudyConfig::default()
    });

    let result = report.variants.first().expect("variant result");
    assert_eq!(result.status, AblationResultStatus::Skipped);
    assert!(
        result
            .reason_codes
            .contains(&ReasonCode::AblationVariantIgnored)
    );
    assert!(
        result
            .flags
            .contains(&AblationInterpretationFlag::UnknownOverrideIgnored)
    );
}

#[test]
fn data_quality_feature_disable_requires_research_only_marker() {
    let base = batch_matrix(
        "ablation-quality-base",
        vec![dataset_entry("valid", "generic_ohlcv_valid.csv", true)],
        vec![baseline_variant("baseline_5m", true)],
    );
    let non_research = AblationRunner::default().run_study(&AblationStudyConfig {
        study_id: "ablation-quality-blocked".to_string(),
        embedded_base_matrix: Some(base.clone()),
        output_root: output_dir("ablation-quality-blocked").display().to_string(),
        variants: vec![AblationVariant {
            variant_id: "disable-data-quality".to_string(),
            dimension: AblationDimension::FeatureGroup,
            overrides: vec![soma_zero::AblationOverride {
                target: "data_quality".to_string(),
                value: AblationValue::Bool(false),
            }],
            research_only: false,
            enabled: true,
            tags: vec![],
            notes: None,
            reason_codes: vec![],
        }],
        ..AblationStudyConfig::default()
    });
    assert_eq!(
        non_research.variants[0].status,
        AblationResultStatus::Skipped
    );

    let research = AblationRunner::default().run_study(&AblationStudyConfig {
        study_id: "ablation-quality-allowed".to_string(),
        embedded_base_matrix: Some(base),
        output_root: output_dir("ablation-quality-allowed").display().to_string(),
        variants: vec![AblationVariant {
            variant_id: "disable-data-quality".to_string(),
            dimension: AblationDimension::FeatureGroup,
            overrides: vec![soma_zero::AblationOverride {
                target: "data_quality".to_string(),
                value: AblationValue::Bool(false),
            }],
            research_only: true,
            enabled: true,
            tags: vec![],
            notes: None,
            reason_codes: vec![],
        }],
        ..AblationStudyConfig::default()
    });
    let result = &research.variants[0];
    assert_eq!(result.status, AblationResultStatus::Applied);
    assert!(
        result
            .reason_codes
            .contains(&ReasonCode::ResearchOnlyOverride)
    );
    assert!(
        result
            .flags
            .contains(&AblationInterpretationFlag::ResearchOnly)
    );
}

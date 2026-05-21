mod common;

use std::path::PathBuf;

use soma_zero::{
    AblationDimension, AblationOverride, AblationRunner, AblationStudyConfig, AblationValue,
    AblationVariant, ablation_report_to_text,
};

use crate::common::{baseline_variant, batch_matrix, dataset_entry, output_dir};

#[test]
fn ablation_runner_is_deterministic_and_writes_outputs() {
    let base = batch_matrix(
        "ablation-runner-base",
        vec![dataset_entry("valid", "generic_ohlcv_valid.csv", true)],
        vec![baseline_variant("baseline_5m", true)],
    );
    let config = AblationStudyConfig {
        study_id: "ablation-runner".to_string(),
        embedded_base_matrix: Some(base),
        output_root: output_dir("ablation-runner").display().to_string(),
        variants: vec![
            AblationVariant {
                variant_id: "volume-off".to_string(),
                dimension: AblationDimension::FeatureGroup,
                overrides: vec![AblationOverride {
                    target: "volume".to_string(),
                    value: AblationValue::Bool(false),
                }],
                research_only: false,
                enabled: true,
                tags: vec![],
                notes: None,
                reason_codes: vec![],
            },
            AblationVariant {
                variant_id: "higher-cost".to_string(),
                dimension: AblationDimension::CostModel,
                overrides: vec![AblationOverride {
                    target: "spread_bps".to_string(),
                    value: AblationValue::Float(6.0),
                }],
                research_only: false,
                enabled: true,
                tags: vec![],
                notes: None,
                reason_codes: vec![],
            },
        ],
        ..AblationStudyConfig::default()
    };

    let runner = AblationRunner::default();
    let first = runner.run_study(&config);
    let second = runner.run_study(&config);
    assert_eq!(
        ablation_report_to_text(&first),
        ablation_report_to_text(&second)
    );

    let report_dir = PathBuf::from(&config.output_root).join(&config.study_id);
    assert!(report_dir.join("ablation_report.json").exists());
    assert!(report_dir.join("ablation_summary.txt").exists());
    assert!(report_dir.join("ablation_summary.md").exists());
    assert!(report_dir.join("sensitivity_summary.txt").exists());
}

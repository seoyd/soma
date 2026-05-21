mod common;

use std::path::PathBuf;
use std::process::Command;

use soma_zero::{
    AblationDimension, AblationOverride, AblationValue, AblationVariant, EvidenceClosureConfig,
    EvidenceClosureRecommendation, EvidenceClosureRunner, ExperimentMatrixConfig, ReasonCode,
    ResearchCampaignConfig,
};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn ablation_report_path() -> PathBuf {
    repo_root()
        .join("target")
        .join("soma_ablations")
        .join("ablation_feature_lab")
        .join("ablation_report.json")
}

fn sprint14_report_path() -> PathBuf {
    repo_root()
        .join("target")
        .join("soma_sprint14")
        .join("sprint14_report.json")
}

fn ensure_source_reports() {
    let ablation_path = ablation_report_path();
    if !ablation_path.exists() {
        let status = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
            .args([
                "ablation",
                "--config",
                "examples/soma_ablation_feature_lab.toml",
            ])
            .status()
            .expect("run ablation");
        assert!(status.success());
    }
    let sprint14_path = sprint14_report_path();
    if !sprint14_path.exists() {
        let status = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
            .args([
                "sprint14",
                "--from-ablation",
                "target/soma_ablations/ablation_feature_lab/ablation_report.json",
                "--out",
                "target/soma_sprint14",
            ])
            .status()
            .expect("run sprint14");
        assert!(status.success());
    }
}

fn closure_config(name: &str) -> EvidenceClosureConfig {
    ensure_source_reports();
    let mut matrix = ExperimentMatrixConfig::from_toml_path(
        &repo_root()
            .join("examples")
            .join("soma_evidence_closure_matrix.toml"),
    )
    .expect("load example matrix");
    matrix.dataset_bundle.output_root = common::output_dir(&format!("{name}-matrix"))
        .display()
        .to_string();
    let closure_root = common::output_dir(&format!("{name}-closure"));
    let evidence_root = common::output_dir(&format!("{name}-evidence"));
    EvidenceClosureConfig {
        closure_id: name.to_string(),
        source_sprint14_report_path: Some(sprint14_report_path().display().to_string()),
        source_ablation_report_path: Some(ablation_report_path().display().to_string()),
        source_campaign_config_path: None,
        embedded_campaign_config: Some(ResearchCampaignConfig {
            campaign_id: format!("{name}-campaign"),
            description: Some(format!("{name} closure campaign")),
            matrix_config_paths: Vec::new(),
            embedded_matrices: vec![matrix],
            output_root: closure_root.join("campaigns").display().to_string(),
            evidence_store_path: evidence_root.display().to_string(),
            run_id: Some(format!("{name}-seed")),
            continue_on_failure: true,
            require_all_matrices_pass: false,
            min_usable_datasets: 2,
            min_total_outcome_records: 20,
            min_regime_coverage_count: 1,
            min_passed_runs: 1,
            min_data_quality_score: 0.80,
            max_allowed_drawdown_regression_pct: 0.02,
            max_allowed_calibration_regression: 0.02,
            max_allowed_risk_governor_instability: 0.15,
            allow_persona_expansion_recommendation: false,
            created_at_ms: Some(42),
            allow_evidence_overwrite: true,
            reason_codes: vec![ReasonCode::DeterministicPath],
            compare_against_campaign_id: None,
            compare_against_report_path: None,
        }),
        output_root: closure_root.display().to_string(),
        evidence_store_path: evidence_root.display().to_string(),
        ablation_variants: vec![
            AblationVariant {
                variant_id: "volume_off".to_string(),
                dimension: AblationDimension::FeatureGroup,
                overrides: vec![AblationOverride {
                    target: "volume".to_string(),
                    value: AblationValue::Bool(false),
                }],
                research_only: false,
                enabled: true,
                tags: vec!["closure".to_string()],
                notes: None,
                reason_codes: vec![ReasonCode::DeterministicPath],
            },
            AblationVariant {
                variant_id: "higher_cost".to_string(),
                dimension: AblationDimension::CostModel,
                overrides: vec![AblationOverride {
                    target: "spread_bps".to_string(),
                    value: AblationValue::Float(4.0),
                }],
                research_only: false,
                enabled: true,
                tags: vec!["closure".to_string()],
                notes: None,
                reason_codes: vec![ReasonCode::DeterministicPath],
            },
        ],
        created_at_ms: Some(42),
        ..EvidenceClosureConfig::default()
    }
}

#[test]
fn runner_handles_missing_source_sprint14_report() {
    let mut config = closure_config("sprint15-missing-source");
    config.source_sprint14_report_path = None;
    let report = EvidenceClosureRunner::default().run_closure(&config);
    assert!(
        report
            .reason_codes
            .contains(&ReasonCode::EvidenceClosureSourceMissing)
    );
    assert_eq!(
        report.final_recommendation,
        EvidenceClosureRecommendation::NeedMoreExperiments
    );
}

#[test]
fn runner_closes_minimum_targets_with_closure_campaign() {
    let config = closure_config("sprint15-closure-success");
    let report = EvidenceClosureRunner::default().run_closure(&config);
    assert!(report.closure_status.all_targets_closed);
    assert_eq!(report.closure_status.usable_dataset_target.added_count, 1);
    assert!(report.added_outcome_summary.additional_outcome_records >= 20);
    assert!(report.added_variant_summary.additional_comparable_variants >= 2);
    assert!(
        report
            .reason_codes
            .contains(&ReasonCode::EvidenceClosureTargetClosed)
    );
    assert_eq!(
        report.final_recommendation,
        EvidenceClosureRecommendation::NeedMoreExperiments
    );
}

#[test]
fn runner_stays_conservative_when_variant_gap_remains() {
    let mut config = closure_config("sprint15-variant-gap");
    config.min_additional_comparable_variants = 3;
    let report = EvidenceClosureRunner::default().run_closure(&config);
    assert!(!report.closure_status.all_targets_closed);
    assert_eq!(report.closure_status.still_missing.comparable_variants, 1);
    assert_eq!(
        report.final_recommendation,
        EvidenceClosureRecommendation::NeedMoreExperiments
    );
}

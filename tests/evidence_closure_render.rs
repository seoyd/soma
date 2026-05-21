mod common;

use std::path::PathBuf;
use std::process::Command;

use soma_zero::{
    EvidenceClosureConfig, EvidenceClosureRunner, ExperimentMatrixConfig, ReasonCode,
    ResearchCampaignConfig, evidence_closure_report_to_markdown, evidence_closure_report_to_text,
};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn build_render_config(name: &str) -> EvidenceClosureConfig {
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
    EvidenceClosureConfig {
        closure_id: name.to_string(),
        source_sprint14_report_path: None,
        source_ablation_report_path: None,
        embedded_campaign_config: Some(ResearchCampaignConfig {
            campaign_id: format!("{name}-campaign"),
            description: Some(format!("{name} closure campaign")),
            matrix_config_paths: Vec::new(),
            embedded_matrices: vec![matrix],
            output_root: closure_root.join("campaigns").display().to_string(),
            evidence_store_path: closure_root.join("evidence").display().to_string(),
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
        evidence_store_path: closure_root.join("evidence").display().to_string(),
        created_at_ms: Some(42),
        ..EvidenceClosureConfig::default()
    }
}

#[test]
fn evidence_closure_rendering_is_deterministic() {
    let config = build_render_config("sprint15-render");
    let report_a = EvidenceClosureRunner::default().run_closure(&config);
    let report_b = EvidenceClosureRunner::default().run_closure(&config);
    assert_eq!(
        evidence_closure_report_to_text(&report_a),
        evidence_closure_report_to_text(&report_b)
    );
    assert_eq!(
        evidence_closure_report_to_markdown(&report_a),
        evidence_closure_report_to_markdown(&report_b)
    );
}

#[test]
fn cli_help_exposes_evidence_close_without_live_commands() {
    let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .arg("--help")
        .output()
        .expect("cli help");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("evidence-close"));
    assert!(!stdout.contains("\n  broker"));
    assert!(!stdout.contains("\n  live"));
    assert!(!stdout.contains("\n  execute"));
}

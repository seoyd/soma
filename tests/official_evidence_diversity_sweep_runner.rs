#[path = "support/sprint48_support.rs"]
mod support;

use std::fs;

use soma_zero::{
    OfficialDiversitySweepConfig, OfficialEvidenceDiversitySweepRunner,
    OfficialEvidenceDiversitySweepStatus, OutcomeDiversityStatus,
};

fn write_all_tp_sweep_files(name: &str) -> std::path::PathBuf {
    let root = support::output_dir(name);
    let plan_path = support::write_file(
        &root,
        "all_tp_sweep_plan.toml",
        &format!(
            "sweep_id = \"{name}-plan\"\nbarrier_profile_registry_path = \"{}\"\nmulti_row_set_config_paths = [\"{}\"]\noutput_root = \"{}\"\nmax_new_rows = 1\nmax_symbols = 2\nmax_timeframes = 2\nmax_horizons = 2\nmax_jobs = 4\nmax_bytes = 500000\nprefer_existing_official_rows = true\nprefer_local_canonical_csv = true\nallow_provider_job_generation = true\nrun_provider_collection_jobs = false\nrun_local_extension_jobs = true\nallow_diagnostic_profiles = false\nallow_exploratory_profiles = false\n",
            support::repo_path("examples/soma_barrier_profiles_primary.toml").display(),
            support::repo_path(
                "examples/sprint48_data/diversity_multi_row_set_all_take_profit.json"
            )
            .display(),
            root.join("plan_outputs").display(),
        ),
    );
    support::write_file(
        &root,
        "all_tp_sweep.toml",
        &format!(
            "run_id = \"{name}\"\nsweep_config_path = \"{}\"\nbarrier_profile_registry_path = \"{}\"\nbatch_outcome_linkage_config_path = \"{}\"\nbatch_counterfactual_completion_config_path = \"{}\"\noutput_root = \"{}\"\nrun_gap_map = true\nrun_row_selector = false\nrun_future_window_scaleout = false\nrun_batch_outcome_linkage = true\nrun_batch_counterfactual_completion = true\nrun_balanced_coverage = true\nrun_sufficiency_v2 = true\nrun_committee_official_benchmark = false\nrun_outcome_coverage = false\nrun_counterfactual_depth_close = false\nrun_core_performance = false\nmax_rows = 5\nmax_symbols = 2\nmax_bytes = 500000\n",
            plan_path.display(),
            support::repo_path("examples/soma_barrier_profiles_primary.toml").display(),
            support::repo_path("examples/sprint48_data/diversity_all_take_profit_outcomes.json")
                .display(),
            support::repo_path(
                "examples/sprint48_data/diversity_all_take_profit_counterfactuals.json"
            )
            .display(),
            root.join("sweep_outputs").display(),
        ),
    )
}

#[test]
fn single_row_example_remains_conservative() {
    let config = support::sweep_single_config("sweep-single");
    let bundle = OfficialEvidenceDiversitySweepRunner::default()
        .run(&config)
        .expect("single sweep bundle");

    assert_eq!(
        bundle.diversity_sweep_report.final_status,
        OfficialEvidenceDiversitySweepStatus::StillNeedMoreOfficialRows
    );
}

#[test]
fn multi_row_all_take_profit_example_remains_single_outcome_dominated() {
    let config_path = write_all_tp_sweep_files("sweep-all-tp");
    let config = soma_zero::OfficialEvidenceDiversitySweepConfig::from_toml_path(&config_path)
        .expect("all tp sweep config");

    let bundle = OfficialEvidenceDiversitySweepRunner::default()
        .run(&config)
        .expect("all tp sweep bundle");

    assert_eq!(
        bundle
            .diversity_sweep_report
            .current_outcome_diversity_status,
        OutcomeDiversityStatus::SingleOutcomeDominated
    );
}

#[test]
fn mixed_outcome_fixture_improves_outcome_diversity() {
    let config = support::sweep_multi_config("sweep-mixed-improves");
    let bundle = OfficialEvidenceDiversitySweepRunner::default()
        .run(&config)
        .expect("mixed sweep bundle");

    assert_eq!(
        bundle
            .diversity_sweep_report
            .previous_outcome_diversity_status,
        Some(OutcomeDiversityStatus::SingleOutcomeDominated)
    );
    assert_eq!(
        bundle
            .diversity_sweep_report
            .current_outcome_diversity_status,
        OutcomeDiversityStatus::HealthyOutcomeDiversity
    );
}

#[test]
fn stop_loss_examples_increase_stop_loss_count() {
    let config = support::sweep_multi_config("sweep-stop-loss");
    let bundle = OfficialEvidenceDiversitySweepRunner::default()
        .run(&config)
        .expect("mixed sweep bundle");

    assert!(bundle.diversity_sweep_report.added_stop_loss > 0);
}

#[test]
fn time_expired_examples_increase_time_expired_count() {
    let config = support::sweep_multi_config("sweep-time-expired");
    let bundle = OfficialEvidenceDiversitySweepRunner::default()
        .run(&config)
        .expect("mixed sweep bundle");

    assert!(bundle.diversity_sweep_report.added_time_expired > 0);
}

#[test]
fn counterfactual_depth_increases_when_outcomes_exist() {
    let config = support::sweep_multi_config("sweep-counterfactual-depth");
    let bundle = OfficialEvidenceDiversitySweepRunner::default()
        .run(&config)
        .expect("mixed sweep bundle");

    assert!(bundle.diversity_sweep_report.added_no_trade_counterfactuals > 0);
    assert!(
        bundle
            .diversity_sweep_report
            .added_risk_denied_counterfactuals
            > 0
    );
}

#[test]
fn committee_benchmark_summary_is_recorded_when_configured() {
    let config = support::sweep_multi_summary_config("sweep-committee-summary");
    let bundle = OfficialEvidenceDiversitySweepRunner::default()
        .run(&config)
        .expect("mixed sweep bundle");

    assert!(bundle.committee_benchmark_summary.is_some());
}

#[test]
fn outcome_coverage_summary_is_recorded_when_configured() {
    let config = support::sweep_multi_summary_config("sweep-outcome-summary");
    let bundle = OfficialEvidenceDiversitySweepRunner::default()
        .run(&config)
        .expect("mixed sweep bundle");

    assert!(bundle.outcome_coverage_summary.is_some());
}

#[test]
fn core_performance_summary_is_recorded_when_configured() {
    let config = support::sweep_multi_summary_config("sweep-core-summary");
    let bundle = OfficialEvidenceDiversitySweepRunner::default()
        .run(&config)
        .expect("mixed sweep bundle");

    assert!(bundle.core_performance_summary.is_some());
}

#[test]
fn runner_does_not_run_provider_jobs_by_default() {
    let sweep = OfficialDiversitySweepConfig::from_toml_path(&support::repo_path(
        "examples/sprint48_data/diversity_sweep_current.toml",
    ))
    .expect("sweep plan");

    assert!(!sweep.run_provider_collection_jobs);
}

#[test]
fn sweep_runner_is_deterministic() {
    let config = support::sweep_multi_config("sweep-deterministic");

    let first = OfficialEvidenceDiversitySweepRunner::default()
        .run(&config)
        .expect("first sweep bundle");
    let second = OfficialEvidenceDiversitySweepRunner::default()
        .run(&config)
        .expect("second sweep bundle");

    assert_eq!(first.final_summary, second.final_summary);
    assert_eq!(
        first.diversity_sweep_report.to_text(),
        second.diversity_sweep_report.to_text()
    );
    let summary_path = config
        .output_dir()
        .join("official_evidence_diversity_summary.txt");
    assert!(fs::metadata(summary_path).is_ok());
}

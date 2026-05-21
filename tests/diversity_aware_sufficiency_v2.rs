#[path = "support/sprint48_support.rs"]
mod support;

use soma_zero::{
    BarrierProfileRegistryBuilder, CommitteeTripleBarrierLabel, DiversityAwareSufficiencyV2Runner,
    DiversityAwareSufficiencyV2Status,
};

#[test]
fn one_row_fails_committee_research_readiness() {
    let mut config = support::sufficiency_config("sufficiency-single-row");
    config.multi_row_set_paths = vec![
        support::repo_path("examples/sprint48_data/diversity_multi_row_set_single.json")
            .display()
            .to_string(),
    ];
    config.batch_outcome_linkage_paths = vec![
        support::repo_path("examples/sprint48_data/diversity_single_outcome.json")
            .display()
            .to_string(),
    ];
    config.batch_counterfactual_completion_paths = vec![
        support::repo_path("examples/sprint48_data/diversity_single_counterfactuals.json")
            .display()
            .to_string(),
    ];

    let report = DiversityAwareSufficiencyV2Runner::default()
        .run(&config)
        .expect("single sufficiency report");

    assert!(!report.passed_committee_benchmark_research);
    assert_eq!(
        report.final_status,
        DiversityAwareSufficiencyV2Status::NeedMoreOfficialRows
    );
}

#[test]
fn two_all_take_profit_rows_remain_plumbing_validated_only() {
    let config = support::sufficiency_config("sufficiency-all-tp");
    let report = DiversityAwareSufficiencyV2Runner::default()
        .run(&config)
        .expect("all tp sufficiency report");

    assert!(report.passed_plumbing_validation);
    assert!(!report.passed_committee_benchmark_research);
    assert_eq!(
        report.final_status,
        DiversityAwareSufficiencyV2Status::PlumbingValidated
    );
}

#[test]
fn missing_time_expired_gate_fails() {
    let mut config = support::sufficiency_config("sufficiency-missing-time-expired");
    config.multi_row_set_paths = vec![
        support::repo_path("examples/sprint48_data/diversity_multi_row_set_mixed.json")
            .display()
            .to_string(),
    ];
    config.batch_outcome_linkage_paths = vec![
        support::repo_path("examples/sprint48_data/diversity_mixed_outcomes.json")
            .display()
            .to_string(),
    ];
    config.batch_counterfactual_completion_paths = vec![
        support::repo_path("examples/sprint48_data/diversity_mixed_counterfactuals.json")
            .display()
            .to_string(),
    ];
    config.min_total_rows = 4;
    config.min_official_complete_rows = 4;
    config.min_symbols = 3;
    config.min_timeframes = 2;
    config.min_horizons = 2;

    let set = support::mixed_set();
    let mut outcomes = support::mixed_outcomes();
    for record in &mut outcomes.records {
        if record.row_id == "nvda-te" {
            record
                .outcome_reference
                .as_mut()
                .expect("outcome ref")
                .triple_barrier_label = CommitteeTripleBarrierLabel::StopLoss;
        }
    }
    let counterfactuals = support::mixed_counterfactuals();
    let registry = BarrierProfileRegistryBuilder::default()
        .build(&support::barrier_profiles_primary(
            "sufficiency-time-expired-registry",
        ))
        .expect("registry");

    let report = DiversityAwareSufficiencyV2Runner::default().run_from_inputs(
        &config,
        &set,
        Some(&outcomes),
        Some(&counterfactuals),
        None,
        None,
        Some(&registry),
    );

    assert_eq!(
        report.final_status,
        DiversityAwareSufficiencyV2Status::PlumbingValidated
    );
    assert!(!report.passed_committee_benchmark_research);
    assert_eq!(
        report.outcome_diversity_status,
        soma_zero::OutcomeDiversityStatus::MissingTimeExpired
    );
}

#[test]
fn single_outcome_domination_gate_fails() {
    let config = support::sufficiency_config("sufficiency-single-outcome");
    let report = DiversityAwareSufficiencyV2Runner::default()
        .run(&config)
        .expect("all tp sufficiency report");

    assert!(
        report
            .failed_gates
            .iter()
            .any(|gate| gate.contains("single_outcome_label_ratio"))
    );
    assert!(!report.passed_committee_benchmark_research);
}

#[test]
fn insufficient_symbols_fail() {
    let mut config = support::sufficiency_config("sufficiency-symbols");
    config.multi_row_set_paths = vec![
        support::repo_path("examples/sprint48_data/diversity_multi_row_set_mixed.json")
            .display()
            .to_string(),
    ];
    config.batch_outcome_linkage_paths = vec![
        support::repo_path("examples/sprint48_data/diversity_mixed_outcomes.json")
            .display()
            .to_string(),
    ];
    config.batch_counterfactual_completion_paths = vec![
        support::repo_path("examples/sprint48_data/diversity_mixed_counterfactuals.json")
            .display()
            .to_string(),
    ];
    config.min_total_rows = 4;
    config.min_official_complete_rows = 4;
    config.min_symbols = 4;
    config.min_timeframes = 2;
    config.min_horizons = 2;
    config.max_single_outcome_label_ratio = 1.0;

    let report = DiversityAwareSufficiencyV2Runner::default()
        .run(&config)
        .expect("symbol sufficiency report");

    assert_eq!(
        report.final_status,
        DiversityAwareSufficiencyV2Status::NeedMoreSymbolDiversity
    );
}

#[test]
fn insufficient_counterfactual_depth_fails() {
    let mut config = support::sufficiency_config("sufficiency-counterfactual-depth");
    config.multi_row_set_paths = vec![
        support::repo_path("examples/sprint48_data/diversity_multi_row_set_mixed.json")
            .display()
            .to_string(),
    ];
    config.batch_outcome_linkage_paths = vec![
        support::repo_path("examples/sprint48_data/diversity_mixed_outcomes.json")
            .display()
            .to_string(),
    ];
    config.batch_counterfactual_completion_paths = vec![
        support::repo_path("examples/sprint48_data/diversity_mixed_counterfactuals.json")
            .display()
            .to_string(),
    ];
    config.min_total_rows = 4;
    config.min_official_complete_rows = 4;
    config.min_symbols = 3;
    config.min_timeframes = 2;
    config.min_horizons = 2;
    config.min_no_trade_counterfactuals = 5;
    config.min_risk_denied_counterfactuals = 5;

    let report = DiversityAwareSufficiencyV2Runner::default()
        .run(&config)
        .expect("counterfactual sufficiency report");

    assert_eq!(
        report.final_status,
        DiversityAwareSufficiencyV2Status::PlumbingValidated
    );
    assert!(!report.passed_committee_benchmark_research);
    assert!(report.failed_gates.iter().any(|gate| {
        gate.contains("no_trade_counterfactual_count 4 < min_no_trade_counterfactuals 5")
    }));
}

#[test]
fn diagnostic_barrier_profiles_cannot_pass_official_sufficiency() {
    let mut config = support::sufficiency_config("sufficiency-diagnostic-registry");
    config.multi_row_set_paths = vec![
        support::repo_path("examples/sprint48_data/diversity_multi_row_set_mixed.json")
            .display()
            .to_string(),
    ];
    config.batch_outcome_linkage_paths = vec![
        support::repo_path("examples/sprint48_data/diversity_mixed_outcomes.json")
            .display()
            .to_string(),
    ];
    config.batch_counterfactual_completion_paths = vec![
        support::repo_path("examples/sprint48_data/diversity_mixed_counterfactuals.json")
            .display()
            .to_string(),
    ];
    config.barrier_profile_registry_path = Some(
        support::repo_path("examples/soma_barrier_profiles_diagnostic.toml")
            .display()
            .to_string(),
    );
    config.min_total_rows = 4;
    config.min_official_complete_rows = 4;
    config.min_symbols = 3;
    config.min_timeframes = 2;
    config.min_horizons = 2;

    let report = DiversityAwareSufficiencyV2Runner::default()
        .run(&config)
        .expect("diagnostic sufficiency report");

    assert_eq!(
        report.final_status,
        DiversityAwareSufficiencyV2Status::DiagnosticOnly
    );
}

#[test]
fn preregistered_primary_profile_can_count() {
    let report = DiversityAwareSufficiencyV2Runner::default()
        .run(&support::sufficiency_config("sufficiency-primary-profile"))
        .expect("primary sufficiency report");

    assert!(report.passed_plumbing_validation);
}

#[test]
fn no_lookahead_ratio_below_threshold_fails() {
    let config = support::sufficiency_config("sufficiency-no-lookahead");
    let mut set = support::all_tp_set();
    set.items[0].no_lookahead_safe = false;
    set.no_lookahead_safe_count = 1;

    let report = DiversityAwareSufficiencyV2Runner::default().run_from_inputs(
        &config,
        &set,
        Some(&support::all_tp_outcomes()),
        Some(&support::all_tp_counterfactuals()),
        None,
        None,
        Some(
            &BarrierProfileRegistryBuilder::default()
                .build(&support::barrier_profiles_primary(
                    "sufficiency-no-lookahead-registry",
                ))
                .expect("registry"),
        ),
    );

    assert!(!report.passed_plumbing_validation);
    assert!(
        report
            .failed_gates
            .iter()
            .any(|gate| gate.contains("no_lookahead_safe_ratio"))
    );
}

#[test]
fn sufficiency_report_is_deterministic() {
    let config = support::sufficiency_config("sufficiency-deterministic");

    let first = DiversityAwareSufficiencyV2Runner::default()
        .run(&config)
        .expect("first sufficiency report");
    let second = DiversityAwareSufficiencyV2Runner::default()
        .run(&config)
        .expect("second sufficiency report");

    assert_eq!(first, second);
    assert_eq!(first.to_text(), second.to_text());
    assert_eq!(first.fingerprint(), second.fingerprint());
}

mod common;
#[path = "support/official_committee_support.rs"]
mod official_committee_support;

use soma_zero::{
    CommitteeOutcomeLinkedComparisonStatus, CommitteeOutcomeLinker, CommitteeReplayReport,
    OfficialCommitteeScenarioPackBuilder, ReasonCode, build_committee_outcome_linked_comparison,
};

#[test]
fn comparison_handles_no_outcomes_no_baselines_and_cost_aware_deltas() {
    let pack_cfg = official_committee_support::controlled_pack_config("comparison-outcome", false);
    let pack = OfficialCommitteeScenarioPackBuilder::default()
        .build(&pack_cfg)
        .expect("pack");
    let pack_dir = common::output_dir("comparison-outcome-pack");
    pack.write_to_dir(&pack_dir).expect("write pack");

    let no_outcomes = CommitteeOutcomeLinker::default()
        .link_from_config(&soma_zero::CommitteeOutcomeLinkerConfig {
            linker_id: "comparison-outcome-none".to_string(),
            scenario_pack_path: Some(
                pack_dir
                    .join("official_scenario_pack.json")
                    .display()
                    .to_string(),
            ),
            output_root: common::output_dir("comparison-outcome-none-out")
                .display()
                .to_string(),
            reason_codes: vec![],
            ..soma_zero::CommitteeOutcomeLinkerConfig::default()
        })
        .expect("link no outcomes");
    let empty_replay = CommitteeReplayReport {
        replay_id: "comparison-outcome".to_string(),
        records: vec![],
        record_count: 0,
        source_summary: "Official".to_string(),
        final_action_counts: std::collections::BTreeMap::new(),
        risk_denial_counts: std::collections::BTreeMap::new(),
        chair_decision_counts: std::collections::BTreeMap::new(),
        deterministic_fingerprint: "fp".to_string(),
        reason_codes: vec![ReasonCode::CommitteeReplayBuilt],
    };
    let no_outcome_report =
        build_committee_outcome_linked_comparison(&no_outcomes, &empty_replay, 1);
    assert_eq!(
        no_outcome_report.comparison_status,
        CommitteeOutcomeLinkedComparisonStatus::NotEnoughOutcomeLinks
    );

    let cfg = soma_zero::CommitteeOutcomeLinkerConfig {
        linker_id: "comparison-linked".to_string(),
        scenario_pack_path: Some(
            pack_dir
                .join("official_scenario_pack.json")
                .display()
                .to_string(),
        ),
        outcome_artifact_paths: vec![
            official_committee_support::write_outcomes("comparison-linked", true)
                .display()
                .to_string(),
        ],
        output_root: common::output_dir("comparison-linked-out")
            .display()
            .to_string(),
        reason_codes: vec![],
        ..soma_zero::CommitteeOutcomeLinkerConfig::default()
    };
    let linked = CommitteeOutcomeLinker::default()
        .link_from_config(&cfg)
        .expect("linked");
    let no_baseline = build_committee_outcome_linked_comparison(&linked, &empty_replay, 1);
    assert_eq!(
        no_baseline.comparison_status,
        CommitteeOutcomeLinkedComparisonStatus::NoBaselineReferences
    );
}

#[test]
fn comparison_includes_no_trade_and_risk_denied_proxies_deterministically() {
    let pack_cfg = official_committee_support::controlled_pack_config("comparison-full", false);
    let pack_cfg_path = official_committee_support::write_pack_config("comparison-full", &pack_cfg);
    let pack = OfficialCommitteeScenarioPackBuilder::default()
        .build(&pack_cfg)
        .expect("pack");
    let pack_dir = common::output_dir("comparison-full-pack");
    pack.write_to_dir(&pack_dir).expect("write");
    let linked = CommitteeOutcomeLinker::default()
        .link_from_config(&official_committee_support::controlled_linker_config(
            "comparison-full",
            &pack_dir.join("official_scenario_pack.json"),
            true,
        ))
        .expect("linked");
    let report = soma_zero::CommitteeOfficialBenchmarkRunner::default()
        .run_bundle(&official_committee_support::controlled_benchmark_config(
            "comparison-full",
            &pack_cfg_path,
            &official_committee_support::write_linker_config(
                "comparison-full",
                &official_committee_support::controlled_linker_config(
                    "comparison-full",
                    &pack_dir.join("official_scenario_pack.json"),
                    true,
                ),
            ),
            false,
        ))
        .expect("bundle");
    let first = build_committee_outcome_linked_comparison(
        &linked,
        &report.committee_benchmark_report.replay_report,
        1,
    );
    let second = build_committee_outcome_linked_comparison(
        &linked,
        &report.committee_benchmark_report.replay_report,
        1,
    );
    assert!(first.no_trade_baseline_counts.contains_key("NoTrade"));
    assert!(first.risk_denied_value_proxy >= 0.0);
    assert_eq!(first.to_text(), second.to_text());
}

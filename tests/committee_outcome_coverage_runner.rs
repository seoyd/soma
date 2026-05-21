mod common;
#[path = "support/official_committee_support.rs"]
mod official_committee_support;

use soma_zero::{
    CommitteeOutcomeCoverageBundleStatus, CommitteeOutcomeCoverageConfig,
    CommitteeOutcomeCoverageRunner,
};

#[test]
fn controlled_runner_is_healthy_and_deterministic() {
    let pack_config = official_committee_support::controlled_pack_config("coverage-runner", false);
    let pack_config_path =
        official_committee_support::write_pack_config("coverage-runner", &pack_config);
    let benchmark_config_path = official_committee_support::write_benchmark_config(
        "coverage-runner",
        &official_committee_support::controlled_benchmark_config(
            "coverage-runner",
            &pack_config_path,
            &official_committee_support::write_linker_config(
                "coverage-runner",
                &official_committee_support::controlled_linker_config(
                    "coverage-runner",
                    &common::output_dir("coverage-runner-pack-ref")
                        .join("official_scenario_pack.json"),
                    true,
                ),
            ),
            false,
        ),
    );
    let coverage_config = official_committee_support::controlled_coverage_config(
        "coverage-runner",
        &benchmark_config_path,
        &pack_config_path,
        &official_committee_support::write_candle_series(
            "coverage-runner",
            "AAPL",
            1_700_000_000_000,
            1.0,
        ),
    );
    let first = CommitteeOutcomeCoverageRunner::default()
        .run(&coverage_config)
        .expect("first");
    let second = CommitteeOutcomeCoverageRunner::default()
        .run(&coverage_config)
        .expect("second");
    assert_eq!(first.to_text(), second.to_text());
    assert_eq!(
        first.final_status,
        CommitteeOutcomeCoverageBundleStatus::OutcomeCoverageHealthy
    );
}

#[test]
fn runner_maps_crypto_research_fixture_and_gaps_conservatively() {
    let crypto = CommitteeOutcomeCoverageRunner::default()
        .run(&CommitteeOutcomeCoverageConfig {
            coverage_id: "coverage-runner-crypto".to_string(),
            official_benchmark_report_paths: vec![
                "examples/soma_committee_official_benchmark_crypto_only.toml".to_string(),
            ],
            output_root: common::output_dir("coverage-runner-crypto-out")
                .display()
                .to_string(),
            allow_crypto_only: true,
            ..CommitteeOutcomeCoverageConfig::default()
        })
        .expect("crypto");
    assert_eq!(
        crypto.final_status,
        CommitteeOutcomeCoverageBundleStatus::CryptoOnly
    );

    let research = CommitteeOutcomeCoverageRunner::default()
        .run(&CommitteeOutcomeCoverageConfig {
            coverage_id: "coverage-runner-research".to_string(),
            scenario_pack_paths: vec![
                official_committee_support::write_pack_config(
                    "coverage-runner-research",
                    &official_committee_support::yfinance_pack_config("coverage-runner-research"),
                )
                .display()
                .to_string(),
            ],
            output_root: common::output_dir("coverage-runner-research-out")
                .display()
                .to_string(),
            allow_yfinance_research: true,
            ..CommitteeOutcomeCoverageConfig::default()
        })
        .expect("research");
    assert_eq!(
        research.final_status,
        CommitteeOutcomeCoverageBundleStatus::ResearchOnly
    );

    let fixture = CommitteeOutcomeCoverageRunner::default()
        .run(&CommitteeOutcomeCoverageConfig {
            coverage_id: "coverage-runner-fixture".to_string(),
            scenario_pack_paths: vec![
                official_committee_support::write_pack_config(
                    "coverage-runner-fixture",
                    &official_committee_support::fixture_pack_config("coverage-runner-fixture"),
                )
                .display()
                .to_string(),
            ],
            output_root: common::output_dir("coverage-runner-fixture-out")
                .display()
                .to_string(),
            allow_fixture: true,
            ..CommitteeOutcomeCoverageConfig::default()
        })
        .expect("fixture");
    assert_eq!(
        fixture.final_status,
        CommitteeOutcomeCoverageBundleStatus::FixtureOnly
    );

    let no_links = CommitteeOutcomeCoverageRunner::default()
        .run(&CommitteeOutcomeCoverageConfig {
            coverage_id: "coverage-runner-no-links".to_string(),
            scenario_pack_paths: vec![
                official_committee_support::write_pack_config(
                    "coverage-runner-no-links",
                    &official_committee_support::controlled_pack_config(
                        "coverage-runner-no-links",
                        false,
                    ),
                )
                .display()
                .to_string(),
            ],
            output_root: common::output_dir("coverage-runner-no-links-out")
                .display()
                .to_string(),
            ..CommitteeOutcomeCoverageConfig::default()
        })
        .expect("no links");
    assert_eq!(
        no_links.final_status,
        CommitteeOutcomeCoverageBundleStatus::NeedMoreOutcomeLinks
    );

    let (no_counterfactuals_pack, no_counterfactuals_linked) =
        official_committee_support::build_controlled_linked_pack(
            "coverage-runner-no-counterfactuals",
            true,
        );
    let no_counterfactuals_pack_dir =
        common::output_dir("coverage-runner-no-counterfactuals-pack-store");
    no_counterfactuals_pack
        .write_to_dir(&no_counterfactuals_pack_dir)
        .expect("write pack");
    let no_counterfactuals_linked_dir =
        common::output_dir("coverage-runner-no-counterfactuals-linked-store");
    let no_counterfactuals_linked_path = no_counterfactuals_linked
        .write_to_dir(&no_counterfactuals_linked_dir)
        .expect("write linked");
    let no_counterfactuals = CommitteeOutcomeCoverageRunner::default()
        .run(&CommitteeOutcomeCoverageConfig {
            coverage_id: "coverage-runner-no-counterfactuals".to_string(),
            outcome_linked_pack_paths: vec![no_counterfactuals_linked_path.display().to_string()],
            scenario_pack_paths: vec![
                no_counterfactuals_pack_dir
                    .join("official_scenario_pack.json")
                    .display()
                    .to_string(),
            ],
            output_root: common::output_dir("coverage-runner-no-counterfactuals-out")
                .display()
                .to_string(),
            ..CommitteeOutcomeCoverageConfig::default()
        })
        .expect("no counterfactuals");
    assert_eq!(
        no_counterfactuals.final_status,
        CommitteeOutcomeCoverageBundleStatus::NeedMoreCounterfactuals
    );
}

mod common;
#[path = "support/official_committee_support.rs"]
mod official_committee_support;

use std::fs;

use soma_zero::CommitteeOutcomeCoverageRunner;

#[test]
fn outcome_coverage_bundle_writes_deterministic_reports() {
    let pack_config =
        official_committee_support::controlled_pack_config("coverage-determinism", false);
    let pack_config_path =
        official_committee_support::write_pack_config("coverage-determinism", &pack_config);
    let benchmark_config_path = official_committee_support::write_benchmark_config(
        "coverage-determinism",
        &official_committee_support::controlled_benchmark_config(
            "coverage-determinism",
            &pack_config_path,
            &official_committee_support::write_linker_config(
                "coverage-determinism",
                &official_committee_support::controlled_linker_config(
                    "coverage-determinism",
                    &common::output_dir("coverage-determinism-pack-ref")
                        .join("official_scenario_pack.json"),
                    true,
                ),
            ),
            false,
        ),
    );
    let config = official_committee_support::controlled_coverage_config(
        "coverage-determinism",
        &benchmark_config_path,
        &pack_config_path,
        &official_committee_support::write_candle_series(
            "coverage-determinism",
            "AAPL",
            1_700_000_000_000,
            1.0,
        ),
    );
    let first = CommitteeOutcomeCoverageRunner::default()
        .run(&config)
        .expect("first");
    let first_text = fs::read_to_string(
        config
            .output_dir()
            .join("committee_outcome_coverage_summary.txt"),
    )
    .expect("summary");
    let second = CommitteeOutcomeCoverageRunner::default()
        .run(&config)
        .expect("second");
    let second_text = fs::read_to_string(
        config
            .output_dir()
            .join("committee_outcome_coverage_summary.txt"),
    )
    .expect("summary");
    assert_eq!(first.to_text(), second.to_text());
    assert_eq!(first_text, second_text);
}

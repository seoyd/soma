mod common;
#[path = "support/official_committee_support.rs"]
mod official_committee_support;

use soma_zero::{CommitteeOfficialBenchmarkRunner, OfficialCommitteeScenarioPackBuilder};

#[test]
fn bundle_contains_pack_link_summary_report_and_is_deterministic() {
    let pack_cfg = official_committee_support::controlled_pack_config("official-bundle", false);
    let pack_cfg_path = official_committee_support::write_pack_config("official-bundle", &pack_cfg);
    let pack = OfficialCommitteeScenarioPackBuilder::default()
        .build(&pack_cfg)
        .expect("pack");
    let pack_dir = common::output_dir("official-bundle-pack");
    pack.write_to_dir(&pack_dir).expect("write");
    let linker_cfg_path = official_committee_support::write_linker_config(
        "official-bundle",
        &official_committee_support::controlled_linker_config(
            "official-bundle",
            &pack_dir.join("official_scenario_pack.json"),
            true,
        ),
    );
    let cfg = official_committee_support::controlled_benchmark_config(
        "official-bundle",
        &pack_cfg_path,
        &linker_cfg_path,
        false,
    );
    let first = CommitteeOfficialBenchmarkRunner::default()
        .run_bundle(&cfg)
        .expect("first");
    let second = CommitteeOfficialBenchmarkRunner::default()
        .run_bundle(&cfg)
        .expect("second");
    assert!(first.official_scenario_pack.row_count() > 0);
    assert!(first.outcome_linked_pack.outcome_linked_count > 0);
    assert!(first.final_summary.contains("final_status="));
    assert_eq!(first.to_text(), second.to_text());
    assert!(!first.to_text().contains("2026-"));
}

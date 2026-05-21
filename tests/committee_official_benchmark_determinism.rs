mod common;
#[path = "support/official_committee_support.rs"]
mod official_committee_support;

use soma_zero::{CommitteeOfficialBenchmarkRunner, OfficialCommitteeScenarioPackBuilder};

#[test]
fn same_controlled_input_produces_same_official_benchmark_output() {
    let pack_cfg =
        official_committee_support::controlled_pack_config("official-benchmark-determinism", false);
    let pack_cfg_path =
        official_committee_support::write_pack_config("official-benchmark-determinism", &pack_cfg);
    let pack = OfficialCommitteeScenarioPackBuilder::default()
        .build(&pack_cfg)
        .expect("pack");
    let pack_dir = common::output_dir("official-benchmark-determinism-pack");
    pack.write_to_dir(&pack_dir).expect("write");
    let linker_cfg_path = official_committee_support::write_linker_config(
        "official-benchmark-determinism",
        &official_committee_support::controlled_linker_config(
            "official-benchmark-determinism",
            &pack_dir.join("official_scenario_pack.json"),
            true,
        ),
    );
    let cfg = official_committee_support::controlled_benchmark_config(
        "official-benchmark-determinism",
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
    assert_eq!(first.audit_summary, second.audit_summary);
    assert_eq!(first.to_text(), second.to_text());
}

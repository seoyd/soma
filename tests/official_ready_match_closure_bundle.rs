mod common;
#[path = "support/sprint44_support.rs"]
mod sprint44_support;

use std::fs;

use soma_zero::OfficialReadyMatchClosureRunner;

#[test]
fn official_ready_match_closure_bundle_writes_deterministic_layout() {
    let mut config = sprint44_support::load_closure_config(
        "examples/soma_official_ready_match_close_official_replication.toml",
    );
    config.output_root = common::output_dir("sprint44-bundle-layout")
        .display()
        .to_string();
    let bundle = OfficialReadyMatchClosureRunner::default()
        .run(&config)
        .expect("bundle");
    let json_path = bundle
        .write_to_dir(&config.output_dir())
        .expect("write bundle");
    assert!(json_path.ends_with("official_ready_match_closure_bundle.json"));
    for file in [
        "match_key_normalization.txt",
        "row_candle_candidates.txt",
        "gap_expansion_consistency.txt",
        "official_candle_lineage.txt",
        "join_repair_plan.txt",
        "official_ready_match_closure.txt",
        "storage_report.txt",
        "official_ready_match_summary.txt",
    ] {
        assert!(
            fs::metadata(config.output_dir().join(file)).is_ok(),
            "missing {file}"
        );
    }
}

mod common;
#[path = "support/official_committee_support.rs"]
mod official_committee_support;

use soma_zero::{ComparableCommitteeEvidenceConfig, ComparableEvidenceBuilder};

#[test]
fn comparable_builder_loads_official_benchmark_bundle_rows() {
    let benchmark_bundle =
        official_committee_support::build_controlled_benchmark_bundle("comparable-builder", true);
    let bundle_dir = common::output_dir("comparable-builder-benchmark");
    let bundle_path = benchmark_bundle
        .write_to_dir(&bundle_dir)
        .expect("write benchmark bundle");

    let config = ComparableCommitteeEvidenceConfig {
        comparable_id: "comparable-builder".to_string(),
        official_committee_benchmark_paths: vec![bundle_path.display().to_string()],
        output_root: common::output_dir("comparable-builder-out")
            .display()
            .to_string(),
        max_rows: 128,
        max_symbols: 3,
        max_bytes: 500_000,
        ..ComparableCommitteeEvidenceConfig::default()
    };

    let comparable = ComparableEvidenceBuilder::default()
        .build(&config)
        .expect("build comparable");
    assert!(!comparable.rows.is_empty());
    assert!(comparable.outcome_reference_count >= 1);
    assert!(comparable.baseline_reference_count >= 1);
}

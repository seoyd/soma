mod common;
#[path = "support/official_committee_support.rs"]
mod official_committee_support;

use soma_zero::{
    OfficialEvidenceReplicationConfig, OfficialEvidenceReplicationRunner,
    OfficialReplicationArtifactInventory,
};

#[test]
fn official_inventory_and_runner_are_deterministic() {
    let csv_path = official_committee_support::write_official_csv_bundle(
        "official-determinism",
        "AAPL",
        32,
        true,
        true,
        true,
    );
    let paths = vec![
        csv_path
            .parent()
            .expect("dir")
            .join("official_provenance.json")
            .display()
            .to_string(),
        csv_path.display().to_string(),
        csv_path
            .parent()
            .expect("dir")
            .join("preflight_report.json")
            .display()
            .to_string(),
    ];
    let first_inventory = OfficialReplicationArtifactInventory::from_paths(&paths);
    let second_inventory = OfficialReplicationArtifactInventory::from_paths(&paths);
    assert_eq!(first_inventory, second_inventory);

    let config = OfficialEvidenceReplicationConfig {
        replication_id: "official-determinism".to_string(),
        output_root: common::output_dir("official-determinism-root")
            .display()
            .to_string(),
        official_canonical_csv_paths: vec![csv_path.display().to_string()],
        max_rows: 1,
        run_official_committee_benchmark: false,
        ..OfficialEvidenceReplicationConfig::default()
    };
    let first = OfficialEvidenceReplicationRunner::default()
        .run(&config)
        .expect("first run");
    let second = OfficialEvidenceReplicationRunner::default()
        .run(&config)
        .expect("second run");
    assert_eq!(first, second);
}

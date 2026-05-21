mod common;
#[path = "support/sprint62_support.rs"]
mod sprint62_support;

use soma_zero::SequenceDatasetExportRunner;

#[test]
fn sequence_dataset_export_is_deterministic() {
    let config = sprint62_support::export_config_from_example(
        "soma_sequence_dataset_export_small.toml",
        "export-deterministic",
    );
    let runner = SequenceDatasetExportRunner::default();
    let left = runner.run(&config).expect("first export");
    let right = runner.run(&config).expect("second export");
    assert_eq!(
        left.sequence_export_manifest.fingerprint,
        right.sequence_export_manifest.fingerprint
    );
    assert_eq!(
        left.sequence_dataset_export_artifact.rows,
        right.sequence_dataset_export_artifact.rows
    );
    assert_eq!(left.split_manifest, right.split_manifest);
}

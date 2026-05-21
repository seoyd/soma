mod common;
#[path = "support/sprint62_support.rs"]
mod sprint62_support;

use soma_zero::{SequenceDatasetExportRunner, SequenceSplitPolicy};

#[test]
fn chronological_holdout_split_works() {
    let config = sprint62_support::export_config_from_example(
        "soma_sequence_dataset_export_small.toml",
        "split-holdout",
    );
    let bundle = SequenceDatasetExportRunner::default()
        .run(&config)
        .expect("run holdout export");
    let split = bundle.split_manifest.expect("split manifest");
    assert_eq!(split.policy, SequenceSplitPolicy::ChronologicalHoldout);
    assert_eq!(split.split_counts.train_count, 3);
    assert_eq!(split.split_counts.validation_count, 1);
    assert_eq!(split.split_counts.test_count, 2);
    assert!(!split.random_seed_used);
}

#[test]
fn walk_forward_split_works() {
    let config = sprint62_support::export_config_from_example(
        "soma_sequence_dataset_export_walk_forward.toml",
        "split-walk-forward",
    );
    let bundle = SequenceDatasetExportRunner::default()
        .run(&config)
        .expect("run walk-forward export");
    let split = bundle.split_manifest.expect("split manifest");
    assert_eq!(split.policy, SequenceSplitPolicy::WalkForward);
    assert!(!split.fold_manifests.is_empty());
}

#[test]
fn export_only_no_split_works() {
    let config = sprint62_support::export_config_from_example(
        "soma_sequence_dataset_export_no_split.toml",
        "split-no-split",
    );
    let bundle = SequenceDatasetExportRunner::default()
        .run(&config)
        .expect("run no-split export");
    let split = bundle.split_manifest.expect("split manifest");
    assert_eq!(split.policy, SequenceSplitPolicy::ExportOnlyNoSplit);
}

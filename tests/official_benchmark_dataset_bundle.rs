mod common;

use std::fs;
use std::path::PathBuf;

use soma_zero::{FeatureName, ReasonCode, build_dataset_bundle};

fn write_dataset_dir(name: &str, rows: &[&str]) -> PathBuf {
    let dir = common::output_dir(name).join("dataset");
    fs::create_dir_all(&dir).expect("create dataset dir");
    let body = rows.join("\n");
    fs::write(
        dir.join("dataset.csv"),
        format!("close,volume,split_kind,fold_id,label_outcome\n{body}\n"),
    )
    .expect("write dataset csv");
    dir
}

#[test]
fn dataset_bundle_includes_feature_schema_and_no_lookahead_summary() {
    let dir = write_dataset_dir(
        "core-benchmark-dataset-safe",
        &[
            "1.0,10.0,Train,0,Win",
            "1.1,11.0,Validation,0,Lose",
            "1.2,12.0,Test,0,Win",
        ],
    );

    let bundle = build_dataset_bundle(&[dir], 1, usize::MAX).expect("build dataset bundle");

    assert_eq!(
        bundle.feature_schema.feature_names,
        vec![FeatureName::Close, FeatureName::Volume]
    );
    assert_eq!(bundle.feature_schema.feature_count, 2);
    assert!(bundle.no_lookahead_report.contains("status=safe"));
}

#[test]
fn dataset_bundle_reason_codes_insufficient_outcomes_and_budget() {
    let dir = write_dataset_dir("core-benchmark-dataset-budget", &["1.0,10.0,Unsafe,0,"]);

    let bundle = build_dataset_bundle(&[dir], 5, 1).expect("build dataset bundle");

    assert!(
        bundle
            .reason_codes
            .contains(&ReasonCode::AiSignalInsufficientOutcomes)
    );
    assert!(bundle.reason_codes.contains(&ReasonCode::BudgetExceeded));
    assert!(bundle.no_lookahead_report.contains("unsafe_rows=1"));
}

#[test]
fn dataset_bundle_is_deterministic_for_same_input() {
    let dir = write_dataset_dir(
        "core-benchmark-dataset-deterministic",
        &["1.0,10.0,Train,0,Win", "1.1,11.0,Test,0,Lose"],
    );

    let first = build_dataset_bundle(&[dir.clone()], 1, usize::MAX).expect("first bundle");
    let second = build_dataset_bundle(&[dir], 1, usize::MAX).expect("second bundle");

    assert_eq!(first, second);
}

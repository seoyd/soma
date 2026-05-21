#[path = "support/sprint69_support.rs"]
mod support;

use std::fs;

use soma_zero::{RealEvidencePredictionImportStatus, RealEvidencePredictionRefreshRunner};

#[test]
fn valid_real_prediction_csv_imports() {
    let config =
        support::sprint75_config_from_example("soma_real_prediction_import.toml", "import");
    let report = RealEvidencePredictionRefreshRunner::default()
        .run_prediction_import(&config)
        .expect("import");
    assert_eq!(
        report.import_status,
        RealEvidencePredictionImportStatus::PredictionImportReady
    );
    assert_eq!(report.imported_rows, 4);
    assert_eq!(report.valid_rows, 4);
    assert_eq!(report.prediction_coverage_ratio, 1.0);
}

#[test]
fn unknown_sequence_ids_are_rejected() {
    let mut config =
        support::sprint75_config_from_example("soma_real_prediction_import.toml", "import-unknown");
    let dir = support::sprint75_output_dir("import-unknown-inputs");
    let path = dir.join("predictions.csv");
    fs::write(
        &path,
        "model_id,model_version,sequence_id,probability,source_class\next-model-b,1.0.0,unknown-seq,0.7,OfficialKIS\n",
    )
    .expect("write csv");
    config.new_prediction_csv_paths = vec![path.display().to_string()];
    let report = RealEvidencePredictionRefreshRunner::default()
        .run_prediction_import(&config)
        .expect("import");
    assert_eq!(
        report.import_status,
        RealEvidencePredictionImportStatus::UnknownSequenceIds
    );
}

#[test]
fn duplicate_sequence_ids_are_rejected() {
    let mut config = support::sprint75_config_from_example(
        "soma_real_prediction_import.toml",
        "import-duplicate",
    );
    let dir = support::sprint75_output_dir("import-duplicate-inputs");
    let path = dir.join("predictions.csv");
    fs::write(
        &path,
        "model_id,model_version,sequence_id,probability,source_class\next-model-b,1.0.0,real-seq-ext-model-b-0001,0.7,OfficialKIS\next-model-b,1.0.0,real-seq-ext-model-b-0001,0.6,OfficialKIS\n",
    )
    .expect("write csv");
    config.new_prediction_csv_paths = vec![path.display().to_string()];
    let report = RealEvidencePredictionRefreshRunner::default()
        .run_prediction_import(&config)
        .expect("import");
    assert_eq!(
        report.import_status,
        RealEvidencePredictionImportStatus::DuplicatePredictions
    );
}

#[test]
fn invalid_probabilities_are_rejected() {
    let mut config =
        support::sprint75_config_from_example("soma_real_prediction_import.toml", "import-invalid");
    let dir = support::sprint75_output_dir("import-invalid-inputs");
    let path = dir.join("predictions.csv");
    fs::write(
        &path,
        "model_id,model_version,sequence_id,probability,source_class\next-model-b,1.0.0,real-seq-ext-model-b-0001,1.4,OfficialKIS\n",
    )
    .expect("write csv");
    config.new_prediction_csv_paths = vec![path.display().to_string()];
    let report = RealEvidencePredictionRefreshRunner::default()
        .run_prediction_import(&config)
        .expect("import");
    assert_eq!(
        report.import_status,
        RealEvidencePredictionImportStatus::InvalidPredictions
    );
}

#[test]
fn missing_model_card_blocks_import() {
    let mut config =
        support::sprint75_config_from_example("soma_real_prediction_import.toml", "import-no-card");
    config.model_card_paths.clear();
    let report = RealEvidencePredictionRefreshRunner::default()
        .run_prediction_import(&config)
        .expect("import");
    assert_eq!(
        report.import_status,
        RealEvidencePredictionImportStatus::MissingModelCard
    );
}

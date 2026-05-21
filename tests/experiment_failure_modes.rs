mod common;

use soma_zero::{ExperimentMode, ExperimentRunner, ExperimentStage, ReasonCode, StageStatus};

#[test]
fn python_unavailable_fails_train_and_compare_safely() {
    let mut config = common::baseline_config("python-unavailable", "generic_ohlcv_valid.csv");
    config.mode = ExperimentMode::TrainAndCompare;
    config.run_python_training = true;
    config.python_executable = Some("definitely-not-a-python-binary".to_string());
    config.training_script_path = Some("local/train_tabular.py".to_string());
    let bundle = ExperimentRunner::default().run(&config);
    assert!(bundle.reason_codes.contains(&ReasonCode::PythonUnavailable));
    assert_eq!(
        bundle.stage_status(ExperimentStage::PythonTrain),
        StageStatus::Failed
    );
}

#[test]
fn bad_csv_fixture_produces_data_unusable_or_failed_validation() {
    let config = common::baseline_config("bad-data", "generic_ohlcv_bad_ohlc.csv");
    let bundle = ExperimentRunner::default().run(&config);
    assert!(
        bundle.reason_codes.contains(&ReasonCode::DataLoadFailed)
            || bundle.reason_codes.contains(&ReasonCode::DataUnusable)
    );
    assert_eq!(
        bundle.stage_status(ExperimentStage::ValidateData),
        StageStatus::Failed
    );
}

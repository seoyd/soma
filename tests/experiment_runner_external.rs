mod common;

use std::fs;

use soma_zero::{ExperimentMode, ExperimentRunner, ExperimentStage, StageStatus};

#[test]
fn external_prediction_only_imports_valid_prediction_csv_and_evaluates() {
    let mut config = common::baseline_config("external-run", "generic_ohlcv_valid.csv");
    config.mode = ExperimentMode::ExternalPredictionOnly;
    let prediction_path = config.output_bundle_dir().join("input_predictions.csv");
    fs::create_dir_all(config.output_bundle_dir()).expect("output dir");
    fs::write(
        &prediction_path,
        common::perfect_prediction_csv("external", "generic_ohlcv_valid.csv"),
    )
    .expect("prediction csv");
    config.prediction_csv_path = Some(prediction_path.display().to_string());

    let bundle = ExperimentRunner::default().run(&config);
    assert!(bundle.external_walk_forward_report.is_some());
    assert_eq!(
        bundle.stage_status(ExperimentStage::ImportPredictions),
        StageStatus::Passed
    );
    assert_eq!(
        bundle.stage_status(ExperimentStage::ExternalEvaluate),
        StageStatus::Passed
    );
}

#[test]
fn invalid_prediction_csv_fails_conservatively_in_strict_mode() {
    let mut config = common::baseline_config("external-invalid", "generic_ohlcv_valid.csv");
    config.mode = ExperimentMode::ExternalPredictionOnly;
    let prediction_path = config.output_bundle_dir().join("input_predictions.csv");
    fs::create_dir_all(config.output_bundle_dir()).expect("output dir");
    fs::write(
        &prediction_path,
        "row_id,symbol,timestamp_ms,timeframe,fold_id,split_kind,model_id,p_win,p_stop,expected_return,expected_drawdown,confidence,no_trade_probability,horizon_bars,reason_codes\nbad,BTCUSDT,1,OneMinute,0,Test,m,9.0,0.1,0.1,0.1,0.8,0.1,8,\n",
    )
    .expect("prediction csv");
    config.prediction_csv_path = Some(prediction_path.display().to_string());

    let bundle = ExperimentRunner::default().run(&config);
    assert!(bundle.external_walk_forward_report.is_none());
    assert_eq!(
        bundle.stage_status(ExperimentStage::ImportPredictions),
        StageStatus::Failed
    );
}

#[test]
fn train_and_compare_can_skip_python_when_prediction_csv_exists() {
    let mut config = common::baseline_config("train-compare-existing", "generic_ohlcv_valid.csv");
    config.mode = ExperimentMode::TrainAndCompare;
    config.run_python_training = false;
    let prediction_path = config.output_bundle_dir().join("input_predictions.csv");
    fs::create_dir_all(config.output_bundle_dir()).expect("output dir");
    fs::write(
        &prediction_path,
        common::perfect_prediction_csv("train_compare", "generic_ohlcv_valid.csv"),
    )
    .expect("prediction csv");
    config.prediction_csv_path = Some(prediction_path.display().to_string());

    let bundle = ExperimentRunner::default().run(&config);
    assert!(bundle.baseline_walk_forward_report.is_some());
    assert!(bundle.external_walk_forward_report.is_some());
    assert!(bundle.model_comparison_report.is_some());
    assert_eq!(
        bundle.stage_status(ExperimentStage::PythonTrain),
        StageStatus::Skipped
    );
}

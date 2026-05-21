mod common;

use std::fs;

use soma_zero::{
    ExperimentMode, ExperimentRunner, experiment::aggregate::build_model_comparison_aggregate,
};

#[test]
fn model_comparison_aggregate_is_conservative_about_external_better() {
    let mut valid_config =
        common::baseline_config("model-compare-valid", "generic_ohlcv_valid.csv");
    valid_config.mode = ExperimentMode::TrainAndCompare;
    valid_config.run_python_training = false;
    let prediction_path = valid_config.output_bundle_dir().join("predictions.csv");
    fs::create_dir_all(valid_config.output_bundle_dir()).expect("output dir");
    fs::write(
        &prediction_path,
        common::perfect_prediction_csv("model_compare", "generic_ohlcv_valid.csv"),
    )
    .expect("prediction csv");
    valid_config.prediction_csv_path = Some(prediction_path.display().to_string());
    let mut valid_bundle = ExperimentRunner::default().run(&valid_config);

    let mut invalid_config =
        common::baseline_config("model-compare-invalid", "generic_ohlcv_valid.csv");
    invalid_config.mode = ExperimentMode::TrainAndCompare;
    invalid_config.run_python_training = false;
    let invalid_prediction_path = invalid_config.output_bundle_dir().join("predictions.csv");
    fs::create_dir_all(invalid_config.output_bundle_dir()).expect("output dir");
    fs::write(
        &invalid_prediction_path,
        "row_id,symbol,timestamp_ms,timeframe,fold_id,split_kind,model_id,p_win\nbad,BTCUSDT,1,OneMinute,0,Test,m,9.0\n",
    )
    .expect("invalid prediction csv");
    invalid_config.prediction_csv_path = Some(invalid_prediction_path.display().to_string());
    let invalid_bundle = ExperimentRunner::default().run(&invalid_config);

    if let Some(report) = valid_bundle.model_comparison_report.as_mut() {
        report.external_better = true;
        report.delta_max_drawdown_pct = 0.10;
    }

    let aggregate = build_model_comparison_aggregate(&[&valid_bundle, &invalid_bundle]);
    assert_eq!(aggregate.compared_runs, 1);
    assert_eq!(aggregate.external_failed_schema_count, 1);
    assert_eq!(aggregate.external_better_count, 0);
    assert_eq!(aggregate.external_missing_prediction_count, 0);
}

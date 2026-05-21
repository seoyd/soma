mod support;

use soma_zero::{CandleExpansionCompileImpactStatus, Sprint89CandleRecoveryRunner};
use support::sprint69_support as sprint;

#[test]
fn candle_compile_impact_stays_sample_backed_without_fake_timings() {
    let config =
        sprint::sprint89_config_from_example("soma_candle_compile_impact.toml", "compile-impact");
    let report = Sprint89CandleRecoveryRunner::default()
        .run_candle_compile_impact(&config)
        .expect("report");
    assert_eq!(
        report.impact_status,
        CandleExpansionCompileImpactStatus::CompileImpactSampleBacked
    );
    assert_eq!(report.target_count_before, Some(2));
    assert_eq!(report.target_count_after, Some(1));
    assert!(!report.measured);
    assert!(report.sample_backed);
    assert!(
        report
            .blocked_targets
            .contains(&"ExternalPrediction".to_string())
    );
}

mod support;

use soma_zero::{CandleExpansionFixtureSetupReductionStatus, Sprint89CandleRecoveryRunner};
use support::sprint69_support as sprint;

#[test]
fn candle_fixture_setup_reduction_uses_shared_harness_and_preserves_determinism() {
    let config = sprint::sprint89_config_from_example(
        "soma_candle_fixture_setup_reduction.toml",
        "candle-fixture-setup",
    );
    let report = Sprint89CandleRecoveryRunner::default()
        .run_candle_fixture_setup_reduction(&config)
        .expect("report");
    assert_eq!(
        report.reduction_status,
        CandleExpansionFixtureSetupReductionStatus::FixtureSetupReduced
    );
    assert_eq!(report.duplicate_json_loads_removed, 1);
    assert_eq!(report.duplicate_toml_loads_removed, 1);
    assert_eq!(report.duplicate_output_dirs_removed, 1);
    assert!(report.shared_fixture_harness_used);
    assert!(report.deterministic_output_preserved);
}

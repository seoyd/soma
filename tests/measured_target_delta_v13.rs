mod support;

use soma_zero::MeasuredTargetDeltaStatusV13;
use support::sprint69_support as sprint;

#[test]
fn measured_target_delta_v13_stays_sample_backed() {
    let report = sprint::run_sprint97_bundle(
        "soma_sprint97_counterfactual_backfill_recover.toml",
        "measured-target-delta-v13",
    )
    .measured_target_delta_report_v13;
    assert_eq!(
        report.delta_status,
        MeasuredTargetDeltaStatusV13::SampleBackedOnly
    );
}

mod support;

use soma_zero::{MeasuredTargetDeltaStatusV11, Sprint95CommitteeCliSafetyRecoveryRunner};
use support::sprint69_support as sprint;

#[test]
fn measured_target_delta_stays_sample_backed_only() {
    let report = Sprint95CommitteeCliSafetyRecoveryRunner::default()
        .run_measured_target_delta_v11(&sprint::sprint95_config_from_example(
            "soma_measured_target_delta_v11.toml",
            "measured-target-delta-v11",
        ))
        .expect("report");
    assert_eq!(
        report.delta_status,
        MeasuredTargetDeltaStatusV11::SampleBackedOnly
    );
    assert_eq!(report.committee_cli_safety_delta, Some(0));
}

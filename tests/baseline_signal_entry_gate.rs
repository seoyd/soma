mod support;

use soma_zero::{
    BaselineSignalEntryGate, BaselineSignalEntryGateStatus,
    Sprint95CommitteeCliSafetyRecoveryRunner,
};
use support::{shared_fixture_harness as harness, sprint69_support as sprint};

#[test]
fn baseline_signal_entry_gate_matches_expected_fixture() {
    let report = Sprint95CommitteeCliSafetyRecoveryRunner::default()
        .run_baseline_signal_entry_gate(&sprint::sprint95_config_from_example(
            "soma_baseline_signal_entry_gate.toml",
            "baseline-signal-entry-gate",
        ))
        .expect("report");
    let mut expected = harness::load_json_fixture::<BaselineSignalEntryGate>(sprint::example_path(
        "sprint95_data/baseline_signal_entry_gate_expected.json",
    ));
    expected.gate_id = report.gate_id.clone();
    assert_eq!(report, expected);
    assert_eq!(
        report.gate_status,
        BaselineSignalEntryGateStatus::BaselineSignalEntryReady
    );
    assert!(report.baseline_signal_next_allowed);
}

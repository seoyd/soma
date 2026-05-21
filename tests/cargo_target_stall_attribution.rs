mod support;

use soma_zero::CargoTargetStallAttributionReport;
use support::sprint111_support::{read_fixture, run_sprint111};

#[test]
fn cargo_target_stall_attribution_matches_fixture() {
    let bundle = run_sprint111(
        "soma_cargo_target_stall_attribution.toml",
        "cargo-target-stall-attribution",
    );
    let expected: CargoTargetStallAttributionReport =
        read_fixture("sprint111_data/target_stall_attribution_expected.json");
    assert_eq!(bundle.cargo_target_stall_attribution_report, expected);
    assert_eq!(
        bundle.cargo_artifact_progress_timeline.timeline_status,
        "ArtifactTimelineReady"
    );
    assert!(
        !bundle
            .cargo_target_stall_attribution_report
            .suspected_stalled_targets
            .is_empty()
    );
}

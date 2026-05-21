mod support;

use soma_zero::{
    CargoTargetProgressTimeline, RealWorkspaceTimeoutAttributionConfig,
    Sprint93TimeoutAttributionRunner,
};
use support::{shared_fixture_harness as harness, sprint69_support as sprint};

fn config(name: &str) -> RealWorkspaceTimeoutAttributionConfig {
    sprint::sprint93_config_from_example("soma_cargo_target_progress_timeline.toml", name)
}

#[test]
fn cargo_target_progress_timeline_matches_expected_fixture() {
    let report = Sprint93TimeoutAttributionRunner::default()
        .run_cargo_target_progress_timeline(&config("cargo-target-progress-timeline"))
        .expect("report");
    let mut expected = harness::load_json_fixture::<CargoTargetProgressTimeline>(
        sprint::example_path("sprint93_data/cargo_target_timeline_expected.json"),
    );
    expected.timeline_id = report.timeline_id.clone();
    assert_eq!(report, expected);
}

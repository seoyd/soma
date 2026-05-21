mod support;

use soma_zero::Sprint93TimeoutAttributionRunner;
use support::{shared_fixture_harness as harness, sprint69_support as sprint};

#[test]
fn control_tower_timeout_attribution_panel_stays_read_only_and_secret_free() {
    let config = sprint::sprint93_config_from_example(
        "soma_control_tower_timeout_attribution.toml",
        "control-tower-timeout-attribution",
    );
    let report = Sprint93TimeoutAttributionRunner::default()
        .run_control_tower_timeout_attribution(&config)
        .expect("report");
    let json = serde_json::to_string(&report).expect("json");
    assert!(json.contains("DashboardRendererEntryReleased"));
    assert!(json.contains("no train button"));
    harness::assert_no_secret_like_values(&json);
}

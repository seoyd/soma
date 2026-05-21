mod support;

use soma_zero::Sprint92KrxWarningClosureRunner;
use support::{shared_fixture_harness as harness, sprint69_support as sprint};

#[test]
fn warning_closure_panel_stays_read_only_and_safe() {
    let config = sprint::sprint92_config_from_example(
        "soma_control_tower_krx_warning_closure.toml",
        "krx-warning-panel",
    );
    let panel = Sprint92KrxWarningClosureRunner::default()
        .run_control_tower_krx_warning_closure(&config)
        .expect("panel");
    let json = serde_json::to_string(&panel).expect("json");
    assert!(json.contains("KrxWarningsClosedWithIsolatedSentinel"));
    assert!(json.contains("DashboardRendererEntryBlockedByUnknownGateCause"));
    assert!(json.contains("no train button"));
    harness::assert_no_secret_like_values(&json);
    assert!(!json.contains("runtime_enabled"));
    assert!(!json.contains("live_trading"));
    assert!(!json.contains("order_id"));
}

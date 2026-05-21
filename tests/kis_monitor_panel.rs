mod common;

use soma_zero::{ControlTowerV1Builder, ControlTowerV1Config};

#[test]
fn kis_monitor_panel_exposes_safe_market_data_readiness_only() {
    let mut config = ControlTowerV1Config::from_toml_path(&common::example_path(
        "soma_control_tower_v1_kis.toml",
    ))
    .expect("config");
    config.output_root = common::sprint54_output_dir("kis-monitor")
        .display()
        .to_string();
    let state = ControlTowerV1Builder::default()
        .build(
            &config,
            Some(&common::example_path("soma_control_tower_v1_kis.toml")),
        )
        .expect("state");
    let panel = state.kis_monitor_panel;
    assert!(panel.auth_ready);
    assert!(panel.base_url_ready);
    assert_eq!(panel.endpoint_policy_status, "MarketDataOnly");
    assert_eq!(panel.collection_plan_status, "DryRunPlanned");
    assert_eq!(panel.candle_sufficiency_status, "MissingFutureWindows");
    assert_eq!(panel.outcome_links, 10);
    assert_eq!(panel.counterfactuals, 6);
    assert!(
        panel
            .next_kis_actions
            .iter()
            .any(|action| action.contains("kis-candle-sufficiency"))
    );
    let json = serde_json::to_string(&panel)
        .expect("json")
        .to_ascii_lowercase();
    assert!(!json.contains("domesticorder"));
    assert!(!json.contains("balance"));
    assert!(!json.contains("account"));
}

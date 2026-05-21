mod common;

use std::fs;

use soma_zero::{
    ProviderPriorityMode, ProviderSimplificationConfig, ProviderSimplificationFinalStatus,
    ProviderSimplificationRunner,
};

fn base_config(name: &str) -> ProviderSimplificationConfig {
    ProviderSimplificationConfig {
        simplification_id: name.to_string(),
        kis_activation_report_paths: vec![
            common::sprint52_data_path("kis_activation_sample.json")
                .display()
                .to_string(),
        ],
        output_root: common::sprint52_output_dir(name).display().to_string(),
        ..ProviderSimplificationConfig::default()
    }
}

#[test]
fn kis_becomes_primary_and_fallback_roles_are_retained() {
    let report = ProviderSimplificationRunner::default().run(&base_config("provider-primary"));
    assert_eq!(report.korean_equity_primary.provider_label, "KIS");
    assert_eq!(
        report.korean_equity_primary.priority_mode,
        ProviderPriorityMode::KISPrimary
    );
    assert_eq!(report.us_equity_primary.provider_label, "KIS");
    assert!(report.fallback_providers.contains(&"KRX".to_string()));
    assert!(
        report
            .fallback_providers
            .contains(&"AlphaVantage".to_string())
    );
    assert!(
        report
            .research_only_providers
            .contains(&"yfinance".to_string())
    );
    assert_eq!(
        report.final_status,
        ProviderSimplificationFinalStatus::KISPrimaryWithKRXReference
    );
}

#[test]
fn unsafe_endpoint_policy_blocks_simplification_without_enabling_trade_surfaces() {
    let output_dir = common::sprint52_output_dir("provider-endpoint-blocked");
    let blocked_path = output_dir.join("kis_blocked.json");
    fs::write(
        &blocked_path,
        r#"{
  "auth_ready": true,
  "endpoint_policy_status": "BlockedBrokerEndpoint",
  "domestic_market_data_ready": true,
  "overseas_market_data_ready": true,
  "realtime_ready": false,
  "blocked_reasons": ["Broker/order/account endpoint detected"],
  "reason_codes": ["KISEndpointDenied", "KISBrokerEndpointDetected"]
}"#,
    )
    .expect("write blocked fixture");
    let report = ProviderSimplificationRunner::default().run(&ProviderSimplificationConfig {
        simplification_id: "provider-endpoint-blocked".to_string(),
        kis_activation_report_paths: vec![blocked_path.display().to_string()],
        output_root: output_dir.display().to_string(),
        ..ProviderSimplificationConfig::default()
    });
    assert_eq!(
        report.final_status,
        ProviderSimplificationFinalStatus::KISPrimaryBlockedByEndpointPolicy
    );
    assert!(
        report
            .warnings
            .iter()
            .any(|warning| warning.contains("broker/order/account"))
    );
}

#[test]
fn provider_simplification_report_is_deterministic() {
    let first = ProviderSimplificationRunner::default()
        .run(&base_config("provider-deterministic"))
        .to_json_string()
        .expect("json");
    let second = ProviderSimplificationRunner::default()
        .run(&base_config("provider-deterministic"))
        .to_json_string()
        .expect("json");
    assert_eq!(first, second);
}

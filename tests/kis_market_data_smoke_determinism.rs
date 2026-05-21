#[path = "support/sprint58_support.rs"]
mod sprint58_support;

use soma_zero::KISMarketDataEvidenceSmokeRunner;

#[test]
fn market_data_smoke_is_deterministic_for_same_fixture_input() {
    sprint58_support::with_kis_env(
        Some("fixture-key"),
        Some("fixture-secret"),
        Some("https://redacted.local"),
        None,
        || {
            let out = sprint58_support::output_dir("kis-market-data-smoke-determinism");
            let config = sprint58_support::smoke_config(&out);
            let first = KISMarketDataEvidenceSmokeRunner::default()
                .run(&config)
                .expect("first smoke");
            let second = KISMarketDataEvidenceSmokeRunner::default()
                .run(&config)
                .expect("second smoke");
            assert_eq!(
                first.market_data_smoke_report,
                second.market_data_smoke_report
            );
            assert_eq!(
                first.control_tower_auto_refresh_report,
                second.control_tower_auto_refresh_report
            );
            assert_eq!(
                first.operational_runbook_v2_report,
                second.operational_runbook_v2_report
            );
        },
    );
}

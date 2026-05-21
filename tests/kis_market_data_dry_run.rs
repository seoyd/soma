#[path = "support/sprint58_support.rs"]
mod sprint58_support;

use soma_zero::{KISEndpointPolicyStatus, KISMarketDataDryRunRunner, KISMarketDataDryRunStatus};

#[test]
fn dry_run_is_ready_with_bounded_fixture_scope() {
    sprint58_support::with_kis_env(
        Some("fixture-key"),
        Some("fixture-secret"),
        Some("https://redacted.local"),
        None,
        || {
            let out = sprint58_support::output_dir("kis-dry-run-ready");
            let report = KISMarketDataDryRunRunner::default()
                .run(&sprint58_support::dry_run_config(&out))
                .expect("dry run");
            assert_eq!(report.dry_run_status, KISMarketDataDryRunStatus::Ready);
            assert_eq!(
                report.endpoint_policy_status,
                KISEndpointPolicyStatus::MarketDataOnly
            );
            assert_eq!(report.planned_domestic_symbols, 1);
            assert_eq!(report.planned_overseas_symbols, 1);
            assert!(!report.safe_to_run_operator_live_collection);
        },
    );
}

#[test]
fn dry_run_is_missing_auth_when_shell_env_is_cleared() {
    sprint58_support::with_kis_env(None, None, None, None, || {
        let out = sprint58_support::output_dir("kis-dry-run-missing");
        let report = KISMarketDataDryRunRunner::default()
            .run(&sprint58_support::dry_run_config(&out))
            .expect("dry run");
        assert_eq!(
            report.dry_run_status,
            KISMarketDataDryRunStatus::MissingAuth
        );
        assert!(
            report
                .blocked_reasons
                .iter()
                .any(|reason| reason.contains("app key"))
        );
    });
}

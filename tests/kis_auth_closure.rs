#[path = "support/sprint58_support.rs"]
mod sprint58_support;

use soma_zero::{KISAuthClosureRunner, KISAuthClosureStatus};

#[test]
fn auth_closure_reports_missing_auth_without_shell_dependency() {
    sprint58_support::with_kis_env(None, None, None, None, || {
        let out = sprint58_support::output_dir("kis-auth-close-missing");
        let report = KISAuthClosureRunner::default()
            .run(&sprint58_support::auth_config(&out))
            .expect("auth closure");
        assert_eq!(
            report.closure_status,
            KISAuthClosureStatus::MissingAppKeyAndSecret
        );
        assert!(!report.safe_for_rest_market_data_dry_run);
    });
}

#[test]
fn auth_closure_redacts_values_when_present() {
    sprint58_support::with_kis_env(
        Some("fixture-key"),
        Some("fixture-secret"),
        Some("https://redacted.local"),
        None,
        || {
            let out = sprint58_support::output_dir("kis-auth-close-ready");
            let report = KISAuthClosureRunner::default()
                .run(&sprint58_support::auth_config(&out))
                .expect("auth closure");
            let text = report.to_text();
            assert_eq!(
                report.closure_status,
                KISAuthClosureStatus::ReadyForDryRunOnly
            );
            assert!(report.safe_for_rest_market_data_dry_run);
            assert!(!text.contains("fixture-key"));
            assert!(!text.contains("fixture-secret"));
            assert!(!text.contains("https://redacted.local"));
        },
    );
}

#[path = "support/sprint58_support.rs"]
mod sprint58_support;

use soma_zero::{
    KISMarketDataEvidenceSmokeFinalStatus, KISMarketDataEvidenceSmokeRunner,
    OperationalRunbookV2FinalStatus, SecretRedactionStatus,
};

#[test]
fn market_data_smoke_builds_refreshable_bundle() {
    sprint58_support::with_kis_env(
        Some("fixture-key"),
        Some("fixture-secret"),
        Some("https://redacted.local"),
        None,
        || {
            let out = sprint58_support::output_dir("kis-market-data-smoke");
            let bundle = KISMarketDataEvidenceSmokeRunner::default()
                .run(&sprint58_support::smoke_config(&out))
                .expect("smoke bundle");
            assert_eq!(
                bundle.market_data_smoke_report.final_status,
                KISMarketDataEvidenceSmokeFinalStatus::KISCompleteRowsImproved
            );
            assert_eq!(
                bundle
                    .control_tower_auto_refresh_report
                    .secret_redaction_report
                    .redaction_status,
                SecretRedactionStatus::Passed
            );
            assert_eq!(
                bundle.operational_runbook_v2_report.final_status,
                OperationalRunbookV2FinalStatus::ReadyToRun
            );
        },
    );
}

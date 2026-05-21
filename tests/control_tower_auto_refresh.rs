#[path = "support/sprint58_support.rs"]
mod sprint58_support;

use soma_zero::{
    ControlTowerAutoRefreshRunner, ControlTowerAutoRefreshStatus,
    KISMarketDataEvidenceSmokeFinalStatus, SecretRedactionStatus,
};

#[test]
fn control_tower_auto_refresh_attaches_smoke_and_secret_state() {
    let out = sprint58_support::output_dir("control-tower-auto-refresh");
    let mut config = sprint58_support::auto_refresh_config(&out);
    let audit_config_path = sprint58_support::write_toml(
        &out.join("secret_redaction_audit.toml"),
        &sprint58_support::secret_audit_config(
            &out,
            vec![
                sprint58_support::sprint58_data_path("secret_redaction_sample_safe.txt")
                    .display()
                    .to_string(),
            ],
        ),
    );
    config.secret_redaction_audit_config_path = Some(audit_config_path.display().to_string());
    let report = ControlTowerAutoRefreshRunner::default()
        .run(&config)
        .expect("auto refresh");
    assert_eq!(
        report.source_smoke_report.final_status,
        KISMarketDataEvidenceSmokeFinalStatus::KISCompleteRowsImproved
    );
    assert_eq!(
        report.secret_redaction_report.redaction_status,
        SecretRedactionStatus::Passed
    );
    assert!(matches!(
        report.refresh_status,
        ControlTowerAutoRefreshStatus::AutoRefreshed
            | ControlTowerAutoRefreshStatus::AutoRefreshedWithWarnings
    ));
}

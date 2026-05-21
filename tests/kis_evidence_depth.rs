mod common;

use std::env;
use std::path::Path;

use soma_zero::{
    KISEvidenceDepthFinalRecommendation, KISEvidenceDepthRunConfig, KISEvidenceDepthRunRunner,
    KISEvidenceDepthStatus,
};

fn with_kis_envs<F: FnOnce()>(f: F) {
    unsafe {
        env::set_var("KIS_APP_KEY", "fixture-key");
        env::set_var("KIS_APP_SECRET", "fixture-secret");
        env::set_var("KIS_BASE_URL", "https://redacted.local");
    }
    f();
    unsafe {
        env::remove_var("KIS_APP_KEY");
        env::remove_var("KIS_APP_SECRET");
        env::remove_var("KIS_BASE_URL");
    }
}

#[test]
fn kis_evidence_depth_aggregates_before_after() {
    with_kis_envs(|| {
        let config_path = Path::new(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/examples/soma_kis_evidence_depth_run.toml"
        ));
        let config = KISEvidenceDepthRunConfig::from_toml_path(config_path).unwrap();
        let bundle = KISEvidenceDepthRunRunner::default()
            .run(&config, Some(config_path))
            .unwrap();
        let report = &bundle.kis_evidence_depth_report;
        assert_eq!(report.official_rows_before, Some(12));
        assert_eq!(report.official_rows_after, 28);
        assert_eq!(report.complete_rows_after, 20);
        assert_eq!(report.outcome_links_after, 12);
        assert_eq!(
            report.depth_status,
            KISEvidenceDepthStatus::NeedFutureWindows
        );
        assert_eq!(
            report.final_recommendation,
            KISEvidenceDepthFinalRecommendation::RunKISCandleSufficiency
        );
        assert!(
            bundle
                .trinity_loop_refresh_summary
                .as_ref()
                .unwrap()
                .loop_ran
        );
        assert!(bundle.control_tower_refresh_report.secret_redaction_passed);
    });
}

#[test]
fn kis_evidence_depth_rejects_remote_paths() {
    let config = KISEvidenceDepthRunConfig {
        kis_activation_report_paths: vec!["https://example.com/report.json".to_string()],
        ..KISEvidenceDepthRunConfig::default()
    };
    assert!(config.validate().is_err());
}

#[test]
fn kis_evidence_depth_bounds_are_enforced() {
    assert!(
        KISEvidenceDepthRunConfig {
            max_rows: 0,
            ..KISEvidenceDepthRunConfig::default()
        }
        .validate()
        .is_err()
    );
    assert!(
        KISEvidenceDepthRunConfig {
            max_symbols: 0,
            ..KISEvidenceDepthRunConfig::default()
        }
        .validate()
        .is_err()
    );
    assert!(
        KISEvidenceDepthRunConfig {
            max_artifacts: 0,
            ..KISEvidenceDepthRunConfig::default()
        }
        .validate()
        .is_err()
    );
    let invalid = "run_id = 'bad'\norder_path = 'forbidden'\n";
    let parsed: Result<KISEvidenceDepthRunConfig, _> = toml::from_str(invalid);
    assert!(parsed.is_err());
}

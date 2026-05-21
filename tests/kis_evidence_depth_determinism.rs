use std::env;
use std::path::Path;

use soma_zero::{KISEvidenceDepthRunConfig, KISEvidenceDepthRunRunner};

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
fn kis_evidence_depth_bundle_is_deterministic() {
    with_kis_envs(|| {
        let config_path = Path::new(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/examples/soma_kis_evidence_depth_run.toml"
        ));
        let config = KISEvidenceDepthRunConfig::from_toml_path(config_path).unwrap();
        let first = KISEvidenceDepthRunRunner::default()
            .run(&config, Some(config_path))
            .unwrap();
        let second = KISEvidenceDepthRunRunner::default()
            .run(&config, Some(config_path))
            .unwrap();
        assert_eq!(
            first.kis_evidence_depth_report.fingerprint,
            second.kis_evidence_depth_report.fingerprint
        );
        assert_eq!(
            first.control_tower_refresh_report.fingerprint,
            second.control_tower_refresh_report.fingerprint
        );
        assert_eq!(
            first.operational_runbook_report.fingerprint,
            second.operational_runbook_report.fingerprint
        );
    });
}

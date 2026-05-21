use std::path::PathBuf;

use soma_zero::{
    KRXOutcomeLinkClosureConfig, KRXOutcomeLinkClosureRunner, KRXOutcomeLinkClosureStatus,
};

fn example_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join(name)
}

#[test]
fn outcome_link_closure_generates_bounded_links_and_counterfactuals() {
    let config = KRXOutcomeLinkClosureConfig::from_toml_path(&example_path(
        "soma_krx_outcome_link_close.toml",
    ))
    .expect("parse outcome config");
    let report = KRXOutcomeLinkClosureRunner::default()
        .run(&config)
        .expect("run outcome closure");
    assert!(report.generated_outcome_links > 0);
    assert!(report.generated_no_trade_counterfactuals > 0);
    assert!(report.generated_risk_denied_counterfactuals > 0);
    assert_eq!(
        report.closure_status,
        KRXOutcomeLinkClosureStatus::KRXCompleteRowsImproved
    );
}

#[test]
fn missing_risk_decisions_keep_risk_denied_zero() {
    let mut config = KRXOutcomeLinkClosureConfig::from_toml_path(&example_path(
        "soma_krx_outcome_link_close.toml",
    ))
    .expect("parse outcome config");
    config
        .official_ready_rows_paths
        .retain(|path| !path.contains("risk"));
    let report = KRXOutcomeLinkClosureRunner::default()
        .run(&config)
        .expect("run outcome closure without risk file");
    assert!(report.generated_outcome_links > 0);
    assert_eq!(report.generated_risk_denied_counterfactuals, 0);
}

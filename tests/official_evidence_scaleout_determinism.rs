#[path = "support/sprint47_support.rs"]
mod sprint47_support;

use soma_zero::OfficialEvidenceScaleOutRunner;

#[test]
fn scaleout_runner_is_deterministic() {
    let config = sprint47_support::scaleout_config("scaleout-determinism");
    let first = OfficialEvidenceScaleOutRunner::default()
        .run(&config)
        .expect("first bundle");
    let second = OfficialEvidenceScaleOutRunner::default()
        .run(&config)
        .expect("second bundle");
    assert_eq!(
        first.scaleout_report.to_text(),
        second.scaleout_report.to_text()
    );
    assert_eq!(first.final_summary, second.final_summary);
}

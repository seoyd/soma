mod common;

use soma_zero::{CommitteeV1RunConfig, CommitteeV1Runner};

#[test]
fn same_fixture_input_produces_same_committee_v1_output() {
    let cfg = CommitteeV1RunConfig {
        run_id: "committee-v1-determinism".to_string(),
        output_root: common::output_dir("committee-v1-determinism")
            .display()
            .to_string(),
        ..CommitteeV1RunConfig::default()
    };
    let first = CommitteeV1Runner::default().run(&cfg).expect("first");
    let second = CommitteeV1Runner::default().run(&cfg).expect("second");
    assert_eq!(first.audit_summary, second.audit_summary);
    assert_eq!(first.to_text(), second.to_text());
}

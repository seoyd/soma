mod common;

use soma_zero::{
    HumanConfirmProtocolConfig, build_owner_decision_impact_report, load_candidate_panel_from_path,
    load_owner_inputs_from_path,
};

#[test]
fn owner_impact_report_counts_expected_actions_and_is_deterministic() {
    let candidates = load_candidate_panel_from_path(&common::sprint53_data_path(
        "candidate_queue_with_owner_items.json",
    ))
    .expect("candidate panel");
    let inputs =
        load_owner_inputs_from_path(&common::sprint53_data_path("owner_inputs_sample.json"))
            .expect("owner inputs");
    let report = build_owner_decision_impact_report(
        "impact-test",
        &candidates,
        &inputs,
        &HumanConfirmProtocolConfig::default(),
    );
    assert!(report.accepted_count >= 3);
    assert!(report.blocked_count >= 1);
    assert!(report.diagnostic_only_count >= 1);
    assert_eq!(report.paper_confirm_count, 1);
    assert_eq!(report.held_count, 1);
    assert_eq!(report.reanalysis_requested_count, 1);
    let second = build_owner_decision_impact_report(
        "impact-test",
        &candidates,
        &inputs,
        &HumanConfirmProtocolConfig::default(),
    );
    assert_eq!(report.fingerprint(), second.fingerprint());
}

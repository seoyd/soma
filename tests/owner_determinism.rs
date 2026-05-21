mod common;

use soma_zero::{
    HumanConfirmProtocolConfig, build_owner_decision_impact_report, build_owner_review_queue,
    load_candidate_panel_from_path, load_owner_inputs_from_path, run_owner_thesis_book,
};

#[test]
fn owner_outputs_are_deterministic_for_same_fixture_input() {
    let candidates = load_candidate_panel_from_path(&common::sprint53_data_path(
        "candidate_queue_with_owner_items.json",
    ))
    .expect("candidates");
    let inputs =
        load_owner_inputs_from_path(&common::sprint53_data_path("owner_inputs_sample.json"))
            .expect("inputs");
    let queue_a = build_owner_review_queue(
        "owner-determinism",
        &candidates,
        &inputs,
        &HumanConfirmProtocolConfig::default(),
    );
    let queue_b = build_owner_review_queue(
        "owner-determinism",
        &candidates,
        &inputs,
        &HumanConfirmProtocolConfig::default(),
    );
    assert_eq!(queue_a.fingerprint(), queue_b.fingerprint());

    let report_a = build_owner_decision_impact_report(
        "owner-determinism",
        &candidates,
        &inputs,
        &HumanConfirmProtocolConfig::default(),
    );
    let report_b = build_owner_decision_impact_report(
        "owner-determinism",
        &candidates,
        &inputs,
        &HumanConfirmProtocolConfig::default(),
    );
    assert_eq!(report_a.fingerprint(), report_b.fingerprint());

    let thesis_config = soma_zero::OwnerThesisBookConfig::from_toml_path(&common::example_path(
        "soma_owner_thesis_book.toml",
    ))
    .expect("thesis config");
    let thesis_a = run_owner_thesis_book(&thesis_config).expect("thesis a");
    let thesis_b = run_owner_thesis_book(&thesis_config).expect("thesis b");
    assert_eq!(thesis_a.fingerprint(), thesis_b.fingerprint());
}

mod common;
#[path = "support/sprint46_support.rs"]
mod sprint46_support;

use std::path::Path;

use soma_zero::{CommitteeTripleBarrierLabel, OutcomeLinkageV3Runner, OutcomeLinkageV3Status};

#[test]
fn outcome_linkage_v3_builds_no_lookahead_safe_outcomes_from_extended_windows() {
    let report = OutcomeLinkageV3Runner::default()
        .run(&sprint46_support::outcome_config("outcome-linkage-v3"))
        .expect("outcome linkage");
    assert_eq!(report.generated_outcome_count, 1);
    assert_eq!(report.official_outcome_count, 1);
    assert_eq!(
        report.records[0]
            .outcome_reference
            .as_ref()
            .unwrap()
            .triple_barrier_label,
        CommitteeTripleBarrierLabel::TakeProfit
    );
    assert!((report.records[0].net_return_pct.unwrap() - 0.0193).abs() < 1e-9);
    assert_eq!(
        report.linkage_status,
        OutcomeLinkageV3Status::OfficialOutcomeLinksImproved
    );
}

#[test]
fn outcome_linkage_expected_fixture_matches_generated_report() {
    let expected = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("examples/sprint46_data/outcome_linkage_expected.json");
    let expected =
        soma_zero::OutcomeLinkageV3Report::from_json_path(&expected).expect("expected report");
    let actual = OutcomeLinkageV3Runner::default()
        .run(&sprint46_support::outcome_config(
            "sprint46-outcome-linkage-v3",
        ))
        .expect("actual report");
    assert_eq!(expected, actual);
}

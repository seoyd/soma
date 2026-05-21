mod support;

use soma_zero::AssertionDestinationProofPlanV1;
use support::sprint115_support::{read_fixture, run_sprint115};

#[test]
fn assertion_destination_proof_plan_v1_matches_expected() {
    let bundle = run_sprint115(
        "soma_assertion_destination_proof_plan_v1.toml",
        "assertion-destination-proof-plan-v1",
    );
    let expected: AssertionDestinationProofPlanV1 =
        read_fixture("sprint115_data/assertion_destination_proof_plan_expected.json");
    assert_eq!(bundle.assertion_destination_proof_plan_v1, expected);
    assert!(
        bundle
            .assertion_destination_proof_plan_v1
            .proof_requirements
            .contains(&"DestinationCapacity".to_string())
    );
}

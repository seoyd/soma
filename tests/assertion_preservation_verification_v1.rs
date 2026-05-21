mod support;

use support::sprint107_support::{read_fixture, run_sprint107};

#[test]
fn assertion_preservation_is_ready() {
    let bundle = run_sprint107(
        "soma_assertion_preservation_verification_v1.toml",
        "assertion-preservation-verification-v1",
    );
    let actual = serde_json::to_value(&bundle.assertion_preservation_verification_report_v1)
        .expect("actual");
    let expected: serde_json::Value =
        read_fixture("sprint107_data/assertion_preservation_expected.json");
    assert_eq!(actual, expected);
    assert_eq!(
        bundle
            .assertion_preservation_verification_report_v1
            .preservation_status,
        "AssertionsPreserved"
    );
}

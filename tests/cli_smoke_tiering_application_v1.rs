mod support;

use support::sprint107_support::{read_fixture, run_sprint107};

#[test]
fn cli_smoke_tiering_preserves_safety_smoke() {
    let bundle = run_sprint107(
        "soma_cli_smoke_tiering_application_v1.toml",
        "cli-smoke-tiering-application-v1",
    );
    let actual =
        serde_json::to_value(&bundle.cli_smoke_tiering_application_report_v1).expect("actual");
    let expected: serde_json::Value =
        read_fixture("sprint107_data/cli_smoke_tiering_application_expected.json");
    assert_eq!(actual, expected);
    assert!(
        bundle
            .cli_smoke_tiering_application_report_v1
            .safety_smoke_preserved
    );
}

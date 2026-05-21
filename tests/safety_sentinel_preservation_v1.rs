mod support;

use support::sprint107_support::{read_fixture, run_sprint107};

#[test]
fn safety_sentinels_remain_preserved() {
    let bundle = run_sprint107(
        "soma_safety_sentinel_preservation_v1.toml",
        "safety-sentinel-preservation-v1",
    );
    let actual =
        serde_json::to_value(&bundle.safety_sentinel_preservation_report_v1).expect("actual");
    let expected: serde_json::Value =
        read_fixture("sprint107_data/safety_sentinel_preservation_expected.json");
    assert_eq!(actual, expected);
    assert!(
        bundle
            .safety_sentinel_preservation_report_v1
            .committee_cli_safety_preserved
    );
    assert!(
        bundle
            .safety_sentinel_preservation_report_v1
            .workspace_determinism_preserved
    );
}

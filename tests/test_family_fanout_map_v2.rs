mod support;

use soma_zero::TestFamilyFanoutMapV2;
use support::sprint111_support::{read_fixture, run_sprint111};

#[test]
fn test_family_fanout_map_v2_matches_fixture() {
    let bundle = run_sprint111(
        "soma_test_family_fanout_map_v2.toml",
        "test-family-fanout-map-v2",
    );
    let expected: TestFamilyFanoutMapV2 =
        read_fixture("sprint111_data/fanout_map_v2_expected.json");
    assert_eq!(bundle.test_family_fanout_map_v2, expected);
    assert_eq!(
        bundle
            .test_family_fanout_map_v2
            .helper_fanout_clusters
            .len(),
        1
    );
    assert_eq!(bundle.test_family_fanout_map_v2.sentinel_clusters.len(), 1);
    assert!(bundle.test_family_fanout_map_v2.sentinel_clusters[0].high_risk);
}

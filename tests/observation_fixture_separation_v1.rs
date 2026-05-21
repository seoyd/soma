mod support;

use support::sprint117_support::run_sprint117;

#[test]
fn observation_fixture_separation_is_explicit() {
    let bundle = run_sprint117(
        "soma_observation_fixture_separation_v1.toml",
        "observation-fixture-separation-v1",
    );
    assert_eq!(
        bundle
            .observation_fixture_separation_report_v1
            .overwritten_actual_count,
        0
    );
    assert_eq!(
        bundle
            .observation_fixture_separation_report_v1
            .separation_status,
        "ObservationSeparationReady"
    );
    assert!(
        bundle
            .observation_fixture_separation_report_v1
            .actual_observation_fields
            .len()
            >= 3
    );
}

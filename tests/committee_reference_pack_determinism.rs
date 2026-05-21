mod common;
#[path = "support/official_committee_support.rs"]
mod official_committee_support;

use soma_zero::CommitteeReferencePackRunner;

#[test]
fn committee_reference_pack_generation_is_deterministic() {
    let config =
        official_committee_support::controlled_reference_pack_config("reference-pack-determinism");
    let first = CommitteeReferencePackRunner::default()
        .build_reference_pack(&config)
        .expect("first");
    let second = CommitteeReferencePackRunner::default()
        .build_reference_pack(&config)
        .expect("second");
    assert_eq!(first, second);
}

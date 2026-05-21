mod common;
#[path = "support/official_committee_support.rs"]
mod official_committee_support;

use soma_zero::{CommitteeReferencePackFinalStatus, CommitteeReferencePackRunner};

#[test]
fn reference_pack_runner_maps_missing_candles_alignment_and_blocks_no_lookahead() {
    let mut no_candles =
        official_committee_support::controlled_reference_pack_config("runner-no-candles");
    no_candles.candle_series_paths.clear();
    let no_candles_bundle = CommitteeReferencePackRunner::default()
        .run(&no_candles)
        .expect("bundle");
    assert_eq!(
        no_candles_bundle.final_status,
        CommitteeReferencePackFinalStatus::NeedMoreCandleData
    );

    let bad_alignment =
        official_committee_support::diagnostics_reference_pack_config("runner-bad-alignment");
    let bad_alignment_bundle = CommitteeReferencePackRunner::default()
        .run(&bad_alignment)
        .expect("bundle");
    assert_eq!(
        bad_alignment_bundle.final_status,
        CommitteeReferencePackFinalStatus::NeedBetterTimestampAlignment
    );
}

#[test]
fn reference_pack_runner_builds_controlled_fixture_and_is_deterministic() {
    let config = official_committee_support::controlled_reference_pack_config("runner-controlled");
    let first = CommitteeReferencePackRunner::default()
        .run(&config)
        .expect("bundle");
    let second = CommitteeReferencePackRunner::default()
        .run(&config)
        .expect("bundle");
    assert_eq!(first.to_text(), second.to_text());
    assert_eq!(first.reference_pack.generated_outcome_count, 3);
    assert!(first.reference_pack.generated_no_trade_count > 0);
}

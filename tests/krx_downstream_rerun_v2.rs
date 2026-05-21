use std::path::PathBuf;

use soma_zero::{
    KRXCandleSufficiencyReport, KRXCanonicalBatchValidationReport, KRXDownstreamRerunV2Summary,
};

fn example_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join(name)
}

#[test]
fn downstream_summary_stays_conservative_when_outcome_links_are_zero() {
    let barrier = example_path("soma_barrier_profiles_primary.toml");
    let batch = KRXCanonicalBatchValidationReport::build(
        "sprint50-batch",
        &vec![
            example_path("sprint50_data/krx_000660_extended_1d.csv")
                .display()
                .to_string(),
        ],
        true,
        true,
    );
    let candle = KRXCandleSufficiencyReport::build(
        "sprint50-candle",
        &vec![
            example_path("sprint50_data/krx_000660_extended_1d.csv")
                .display()
                .to_string(),
        ],
        Some(barrier.to_str().expect("barrier path")),
    );
    let summary = KRXDownstreamRerunV2Summary::build(&batch, &candle, None, true, true);
    assert_eq!(summary.outcome_links_after, None);
    assert_eq!(
        summary.committee_status_after.as_deref(),
        Some("ConservativeBlockedMissingOutcomeLinks")
    );
    assert_eq!(
        summary.core_status_after.as_deref(),
        Some("CoreBlockedByOfficialData")
    );
}

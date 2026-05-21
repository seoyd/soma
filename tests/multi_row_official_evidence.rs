#[path = "support/sprint47_support.rs"]
mod sprint47_support;

use soma_zero::{
    ComparableEvidenceSourceClass, MultiRowOfficialEvidenceSetBuilder,
    MultiRowOfficialEvidenceSetConfig, MultiRowOfficialEvidenceStatus,
};

#[test]
fn config_validation_is_conservative() {
    let mut config = MultiRowOfficialEvidenceSetConfig::default();
    config.official_ready_inventory_paths = vec!["https://example.com/inventory.json".to_string()];
    assert!(config.validate().is_err());

    let mut max_rows = MultiRowOfficialEvidenceSetConfig::default();
    max_rows.max_rows = 1001;
    assert!(max_rows.validate().is_err());

    let mut max_symbols = MultiRowOfficialEvidenceSetConfig::default();
    max_symbols.max_symbols = 11;
    assert!(max_symbols.validate().is_err());

    let mut max_timeframes = MultiRowOfficialEvidenceSetConfig::default();
    max_timeframes.max_timeframes = 6;
    assert!(max_timeframes.validate().is_err());

    let mut max_horizons = MultiRowOfficialEvidenceSetConfig::default();
    max_horizons.max_horizons = 6;
    assert!(max_horizons.validate().is_err());
    assert!(!MultiRowOfficialEvidenceSetConfig::default().allow_yfinance_research);
    assert!(!MultiRowOfficialEvidenceSetConfig::default().allow_fixture);
    assert!(!MultiRowOfficialEvidenceSetConfig::default().allow_controlled_diagnostic);
}

#[test]
fn single_and_multi_row_sets_are_counted_deterministically() {
    let single = MultiRowOfficialEvidenceSetBuilder::default()
        .build(&sprint47_support::example_multi_row_set(
            "multi-row-single",
            "examples/soma_multi_row_official_set_single_row.toml",
        ))
        .expect("single-row set");
    assert_eq!(single.official_complete_rows, 1);
    assert_eq!(
        single.status,
        MultiRowOfficialEvidenceStatus::OfficialComplete
    );

    let first = MultiRowOfficialEvidenceSetBuilder::default()
        .build(&sprint47_support::example_multi_row_set(
            "multi-row-multi-a",
            "examples/soma_multi_row_official_set_multi_row.toml",
        ))
        .expect("first set");
    let second = MultiRowOfficialEvidenceSetBuilder::default()
        .build(&sprint47_support::example_multi_row_set(
            "multi-row-multi-b",
            "examples/soma_multi_row_official_set_multi_row.toml",
        ))
        .expect("second set");
    assert_eq!(first.official_complete_rows, 2);
    assert_eq!(first.symbol_count, 2);
    assert_eq!(first.to_text(), second.to_text());
}

#[test]
fn controlled_rows_remain_diagnostic_only() {
    let set = MultiRowOfficialEvidenceSetBuilder::default()
        .build(&sprint47_support::example_multi_row_set(
            "multi-row-controlled",
            "examples/soma_multi_row_official_set_controlled.toml",
        ))
        .expect("controlled set");
    assert_eq!(set.official_complete_rows, 0);
    assert!(
        set.items
            .iter()
            .all(|item| item.source_class == ComparableEvidenceSourceClass::ControlledDiagnostic)
    );
    assert!(set.controlled_rows >= 1);
}

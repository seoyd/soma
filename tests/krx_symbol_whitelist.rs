use std::path::PathBuf;

use soma_zero::{KRXSymbolEntry, KRXSymbolWhitelistConfig, ProviderMarket, ReasonCode};

fn example_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join(name)
}

fn symbol(provider_symbol: &str, enabled: bool) -> KRXSymbolEntry {
    KRXSymbolEntry {
        provider_symbol: provider_symbol.to_string(),
        normalized_symbol: String::new(),
        market: ProviderMarket::KoreanEquity,
        venue: Some("KRX".to_string()),
        display_name: None,
        enabled,
        max_rows: Some(120),
        timeframe: "1d".to_string(),
        reason_codes: Vec::new(),
    }
}

#[test]
fn compact_whitelist_example_parses() {
    let config = KRXSymbolWhitelistConfig::from_toml_path(&example_path(
        "soma_krx_symbol_whitelist_compact.toml",
    ))
    .expect("parse whitelist example");
    config.validate().expect("validate whitelist example");
    let whitelist = config.build();
    assert_eq!(whitelist.whitelist_id, "krx_whitelist_compact");
    assert_eq!(whitelist.enabled_entries, vec!["000660", "005930"]);
}

#[test]
fn wildcard_and_invalid_symbols_are_skipped() {
    let config = KRXSymbolWhitelistConfig {
        whitelist_id: "wildcard-denied".to_string(),
        symbols: vec![
            symbol("005930", true),
            symbol("ALL", true),
            symbol("ABC", true),
        ],
        ..KRXSymbolWhitelistConfig::default()
    };
    let whitelist = config.build();
    assert!(
        whitelist
            .reason_codes
            .contains(&ReasonCode::DeniedByDefault)
    );
    assert_eq!(whitelist.enabled_entries, vec!["005930"]);
    assert_eq!(whitelist.skipped_entries, vec!["ABC", "ALL"]);
    assert_eq!(whitelist.entries[0].provider_symbol, "005930");
    assert_eq!(whitelist.entries[0].normalized_symbol, "005930");
}

#[test]
fn max_symbols_and_disabled_entries_are_handled_deterministically() {
    let config = KRXSymbolWhitelistConfig {
        whitelist_id: "budgeted".to_string(),
        max_symbols: 2,
        symbols: vec![
            symbol("005930", true),
            symbol("000660", true),
            symbol("035420", true),
            symbol("051910", false),
        ],
        ..KRXSymbolWhitelistConfig::default()
    };
    let first = config.build();
    let second = config.build();
    assert_eq!(first, second);
    assert!(first.reason_codes.contains(&ReasonCode::BudgetExceeded));
    assert!(first.skipped_entries.contains(&"051910".to_string()));
    assert_eq!(
        first
            .entries
            .iter()
            .map(|entry| entry.normalized_symbol.clone())
            .collect::<Vec<_>>(),
        vec!["000660", "005930", "035420", "051910"]
    );
}

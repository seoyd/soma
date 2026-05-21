use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::core::ReasonCode;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum AssetClass {
    Spot,
    Crypto,
    Equity,
    Stock,
    Forex,
    Futures,
    Etf,
    Unknown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum MarketVenue {
    Generic,
    Binance,
    Upbit,
    KRX,
    KOSPI,
    KOSDAQ,
    NASDAQ,
    NYSE,
    AMEX,
    US,
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SymbolSpec {
    pub raw_symbol: String,
    pub normalized_symbol: String,
    pub asset_class: AssetClass,
    pub venue: MarketVenue,
    pub quote_currency: Option<String>,
    pub base_currency: Option<String>,
    pub tick_size: Option<String>,
    pub lot_size: Option<String>,
    pub timezone_hint: Option<String>,
    pub reason_codes: Vec<ReasonCode>,
}

impl SymbolSpec {
    pub fn new(raw_symbol: impl Into<String>, venue: MarketVenue, asset_class: AssetClass) -> Self {
        let raw_symbol = raw_symbol.into();
        let normalized_symbol = normalize_symbol_value(&raw_symbol);
        let (base_currency, quote_currency) = split_pair(&raw_symbol);
        let mut reason_codes = Vec::new();
        if normalized_symbol.is_empty() {
            reason_codes.push(ReasonCode::InvalidSymbol);
        } else {
            reason_codes.push(ReasonCode::SymbolNormalized);
        }
        Self {
            raw_symbol,
            normalized_symbol,
            asset_class,
            venue,
            quote_currency,
            base_currency,
            tick_size: None,
            lot_size: None,
            timezone_hint: None,
            reason_codes,
        }
    }

    pub fn guessed(raw_symbol: impl Into<String>, venue: MarketVenue) -> Self {
        let venue = match venue {
            MarketVenue::Unknown => MarketVenue::Generic,
            value => value,
        };
        let asset_class = match venue {
            MarketVenue::Binance | MarketVenue::Upbit => AssetClass::Crypto,
            MarketVenue::KRX
            | MarketVenue::KOSPI
            | MarketVenue::KOSDAQ
            | MarketVenue::US
            | MarketVenue::NASDAQ
            | MarketVenue::NYSE
            | MarketVenue::AMEX => AssetClass::Equity,
            _ => AssetClass::Unknown,
        };
        Self::new(raw_symbol, venue, asset_class)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SymbolRegistry {
    entries: BTreeMap<String, SymbolSpec>,
}

impl SymbolRegistry {
    pub fn register_symbol(&mut self, mut spec: SymbolSpec) -> Result<SymbolSpec, Vec<ReasonCode>> {
        let validation = self.validate_symbol(&spec.raw_symbol);
        if !validation.is_empty() {
            return Err(validation);
        }
        spec.normalized_symbol = self.normalize_symbol(&spec.raw_symbol);
        spec.reason_codes.push(ReasonCode::SymbolRegistered);
        self.entries
            .insert(spec.normalized_symbol.clone(), spec.clone());
        Ok(spec)
    }

    pub fn normalize_symbol(&self, raw_symbol: &str) -> String {
        normalize_symbol_value(raw_symbol)
    }

    pub fn lookup_symbol(&self, raw_symbol: &str) -> Option<&SymbolSpec> {
        let normalized = self.normalize_symbol(raw_symbol);
        self.entries.get(&normalized)
    }

    pub fn validate_symbol(&self, raw_symbol: &str) -> Vec<ReasonCode> {
        let normalized = self.normalize_symbol(raw_symbol);
        if normalized.is_empty() {
            vec![ReasonCode::InvalidSymbol]
        } else {
            Vec::new()
        }
    }
}

fn normalize_symbol_value(raw_symbol: &str) -> String {
    raw_symbol
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .flat_map(char::to_uppercase)
        .collect()
}

fn split_pair(raw_symbol: &str) -> (Option<String>, Option<String>) {
    let tokens = raw_symbol
        .split(['-', '_', '/'])
        .filter(|token| !token.is_empty())
        .map(|token| token.to_ascii_uppercase())
        .collect::<Vec<_>>();
    match tokens.as_slice() {
        [base, quote] => (Some(base.clone()), Some(quote.clone())),
        _ => (None, None),
    }
}

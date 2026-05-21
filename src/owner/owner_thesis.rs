use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_hash_string, stable_ordered_strings, stable_reason_codes};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum OwnerThesisType {
    Bullish,
    Bearish,
    #[default]
    Neutral,
    RiskWarning,
    DataQualityWarning,
    EventNote,
    WatchlistRationale,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct OwnerThesisNote {
    pub thesis_id: String,
    #[serde(default)]
    pub symbol: Option<String>,
    #[serde(default)]
    pub market: Option<String>,
    #[serde(default)]
    pub timeframe: Option<String>,
    pub thesis_type: OwnerThesisType,
    pub text: String,
    #[serde(default)]
    pub structured_tags: Vec<String>,
    #[serde(default)]
    pub evidence_links: Option<Vec<String>>,
    #[serde(default)]
    pub expires_at_timestamp_ms: Option<u64>,
    pub active: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct OwnerThesisBook {
    #[serde(default)]
    pub active_notes: Vec<OwnerThesisNote>,
    #[serde(default)]
    pub expired_notes: Vec<OwnerThesisNote>,
    #[serde(default)]
    pub notes_by_symbol: BTreeMap<String, Vec<OwnerThesisNote>>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl OwnerThesisNote {
    pub fn stabilize(&mut self) {
        self.structured_tags = stable_ordered_strings(&self.structured_tags);
        if let Some(links) = &mut self.evidence_links {
            *links = stable_ordered_strings(links);
        }
        self.reason_codes = stable_reason_codes(&self.reason_codes);
    }

    pub fn is_signal(&self) -> bool {
        false
    }
}

impl OwnerThesisBook {
    pub fn from_notes(notes: &[OwnerThesisNote], timestamp_ms: Option<u64>) -> Self {
        let now = timestamp_ms.unwrap_or_default();
        let mut book = OwnerThesisBook {
            active_notes: Vec::new(),
            expired_notes: Vec::new(),
            notes_by_symbol: BTreeMap::new(),
            reason_codes: vec![ReasonCode::OwnerThesisBookBuilt],
        };
        for mut note in notes.to_vec() {
            note.active = note.active
                && note
                    .expires_at_timestamp_ms
                    .is_none_or(|expiry| expiry >= now);
            note.stabilize();
            if note.active {
                if let Some(symbol) = &note.symbol {
                    book.notes_by_symbol
                        .entry(symbol.clone())
                        .or_default()
                        .push(note.clone());
                }
                book.active_notes.push(note);
            } else {
                book.expired_notes.push(note);
            }
        }
        book.stabilize();
        book
    }

    pub fn stabilize(&mut self) {
        self.active_notes
            .sort_by(|left, right| left.thesis_id.cmp(&right.thesis_id));
        self.expired_notes
            .sort_by(|left, right| left.thesis_id.cmp(&right.thesis_id));
        for notes in self.notes_by_symbol.values_mut() {
            notes.sort_by(|left, right| left.thesis_id.cmp(&right.thesis_id));
            for note in notes {
                note.stabilize();
            }
        }
        self.reason_codes = stable_reason_codes(&self.reason_codes);
    }

    pub fn fingerprint(&self) -> String {
        let mut copy = self.clone();
        copy.stabilize();
        stable_hash_string(&serde_json::to_string(&copy).unwrap_or_default())
    }

    pub fn to_text(&self) -> String {
        [
            "research_only_warning=owner thesis notes are diagnostics, not signals".to_string(),
            "paper_only_warning=thesis notes can inform paper review but never create live execution".to_string(),
            format!("active_notes={}", self.active_notes.len()),
            format!("expired_notes={}", self.expired_notes.len()),
            format!(
                "symbols={}",
                self.notes_by_symbol.keys().cloned().collect::<Vec<_>>().join("|")
            ),
            format!("fingerprint={}", self.fingerprint()),
        ]
        .join("\n")
    }
}

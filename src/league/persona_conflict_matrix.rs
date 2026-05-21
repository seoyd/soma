use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::core::ReasonCode;

use super::committee_replay::CommitteeReplayReport;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PersonaConflictPair {
    pub persona_a: String,
    pub persona_b: String,
    pub same_stance_count: usize,
    pub opposite_stance_count: usize,
    pub disagreement_rate: f64,
    pub average_conviction_delta: f64,
    pub high_conflict_count: usize,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PersonaConflictStatus {
    HealthyDiversity,
    TooAligned,
    TooConflicted,
    InsufficientSamples,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PersonaConflictMatrix {
    pub pairs: Vec<PersonaConflictPair>,
    pub most_aligned_pairs: Vec<String>,
    pub most_conflicted_pairs: Vec<String>,
    pub average_disagreement: f64,
    pub groupthink_frequency: f64,
    pub conflict_status: PersonaConflictStatus,
    pub reason_codes: Vec<ReasonCode>,
}

pub fn build_persona_conflict_matrix(report: &CommitteeReplayReport) -> PersonaConflictMatrix {
    let mut raw = BTreeMap::<(String, String), Vec<(f64, f64)>>::new();
    let mut groupthink_count = 0usize;
    for record in &report.records {
        let votes = &record.persona_votes;
        if votes.iter().all(|vote| vote.stance.score() >= 0.0)
            || votes.iter().all(|vote| vote.stance.score() <= 0.0)
        {
            groupthink_count += 1;
        }
        for index in 0..votes.len() {
            for other in (index + 1)..votes.len() {
                let left = &votes[index];
                let right = &votes[other];
                let key = if left.persona_id <= right.persona_id {
                    (left.persona_id.clone(), right.persona_id.clone())
                } else {
                    (right.persona_id.clone(), left.persona_id.clone())
                };
                raw.entry(key)
                    .or_default()
                    .push((left.stance.score(), right.stance.score()));
            }
        }
    }
    let mut pairs = raw
        .into_iter()
        .map(|((persona_a, persona_b), rows)| {
            let sample_count = rows.len().max(1) as f64;
            let same_stance_count = rows
                .iter()
                .filter(|(left, right)| (*left - *right).abs() < 0.001)
                .count();
            let opposite_stance_count = rows
                .iter()
                .filter(|(left, right)| {
                    left.signum() != right.signum() && *left != 0.0 && *right != 0.0
                })
                .count();
            let disagreement_rate = rows
                .iter()
                .filter(|(left, right)| (*left - *right).abs() >= 0.001)
                .count() as f64
                / sample_count;
            let average_conviction_delta = rows
                .iter()
                .map(|(left, right)| (left - right).abs())
                .sum::<f64>()
                / sample_count;
            let high_conflict_count = rows
                .iter()
                .filter(|(left, right)| (left - right).abs() >= 1.0)
                .count();
            PersonaConflictPair {
                persona_a,
                persona_b,
                same_stance_count,
                opposite_stance_count,
                disagreement_rate,
                average_conviction_delta,
                high_conflict_count,
                reason_codes: vec![ReasonCode::PersonaConflictMatrixBuilt],
            }
        })
        .collect::<Vec<_>>();
    pairs.sort_by(|left, right| {
        left.persona_a
            .cmp(&right.persona_a)
            .then(left.persona_b.cmp(&right.persona_b))
    });
    let average_disagreement = if pairs.is_empty() {
        0.0
    } else {
        pairs.iter().map(|pair| pair.disagreement_rate).sum::<f64>() / pairs.len() as f64
    };
    let groupthink_frequency = groupthink_count as f64 / report.records.len().max(1) as f64;
    let conflict_status = if report.records.len() < 3 {
        PersonaConflictStatus::InsufficientSamples
    } else if groupthink_frequency >= 0.75 {
        PersonaConflictStatus::TooAligned
    } else if average_disagreement >= 0.75 {
        PersonaConflictStatus::TooConflicted
    } else {
        PersonaConflictStatus::HealthyDiversity
    };
    let mut most_aligned_pairs = pairs
        .iter()
        .filter(|pair| pair.same_stance_count >= pair.opposite_stance_count)
        .map(|pair| format!("{}::{}", pair.persona_a, pair.persona_b))
        .collect::<Vec<_>>();
    let mut most_conflicted_pairs = pairs
        .iter()
        .filter(|pair| pair.opposite_stance_count > 0 || pair.high_conflict_count > 0)
        .map(|pair| format!("{}::{}", pair.persona_a, pair.persona_b))
        .collect::<Vec<_>>();
    most_aligned_pairs.sort();
    most_conflicted_pairs.sort();
    PersonaConflictMatrix {
        pairs,
        most_aligned_pairs,
        most_conflicted_pairs,
        average_disagreement,
        groupthink_frequency,
        conflict_status,
        reason_codes: vec![ReasonCode::PersonaConflictMatrixBuilt],
    }
}

impl PersonaConflictMatrix {
    pub fn to_text(&self) -> String {
        let mut lines = vec![
            format!("conflict_status={:?}", self.conflict_status),
            format!("average_disagreement={:.6}", self.average_disagreement),
            format!("groupthink_frequency={:.6}", self.groupthink_frequency),
        ];
        for pair in &self.pairs {
            lines.push(format!(
                "pair={}::{};same={};opposite={};disagreement_rate={:.6}",
                pair.persona_a,
                pair.persona_b,
                pair.same_stance_count,
                pair.opposite_stance_count,
                pair.disagreement_rate
            ));
        }
        lines.join("\n")
    }
}

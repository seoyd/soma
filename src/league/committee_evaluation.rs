use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::core::ReasonCode;

use super::committee_risk_bridge::{CommitteeFinalAction, CommitteeOutcome};
use super::persona_vote::PersonaStance;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PersonaEvaluationMetric {
    pub persona_id: String,
    pub sample_count: usize,
    pub stance_counts: BTreeMap<String, usize>,
    pub avg_conviction: f64,
    pub avg_voice_power: f64,
    #[serde(default)]
    pub calibration_proxy: Option<f64>,
    #[serde(default)]
    pub no_trade_value_proxy: Option<f64>,
    pub risk_alignment_score: f64,
    pub doctrine_violation_count: usize,
    pub overtrade_warning: bool,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommitteeEvaluationRecommendation {
    NotEnoughSamples,
    KeepCurrentPersonas,
    ImprovePersonaThresholds,
    ImproveChairWeights,
    ConsiderSixPersonaDesignReview,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CommitteeEvaluationScaffold {
    pub persona_metrics: Vec<PersonaEvaluationMetric>,
    pub chair_metrics: Vec<String>,
    pub risk_metrics: Vec<String>,
    pub enough_samples: bool,
    pub recommendation: CommitteeEvaluationRecommendation,
    pub reason_codes: Vec<ReasonCode>,
}

pub fn build_committee_evaluation_scaffold(
    outcomes: &[CommitteeOutcome],
) -> CommitteeEvaluationScaffold {
    let mut by_persona = BTreeMap::<String, Vec<_>>::new();
    for outcome in outcomes {
        for vote in &outcome.committee_record.all_votes {
            by_persona
                .entry(vote.persona_id.clone())
                .or_default()
                .push((vote, outcome));
        }
    }
    let mut persona_metrics = by_persona
        .into_iter()
        .map(|(persona_id, rows)| {
            let sample_count = rows.len();
            let mut stance_counts = BTreeMap::new();
            let mut conviction_sum = 0.0;
            let mut voice_sum = 0.0;
            let mut doctrine_violation_count = 0usize;
            let mut risk_align = 0.0;
            let mut no_trade_count = 0usize;
            for (vote, outcome) in &rows {
                *stance_counts
                    .entry(format!("{:?}", vote.stance))
                    .or_insert(0) += 1;
                conviction_sum += vote.conviction;
                voice_sum += vote.voice_power;
                doctrine_violation_count += vote.doctrine_violations.len();
                if matches!(
                    outcome.final_action,
                    CommitteeFinalAction::FinalDenied | CommitteeFinalAction::FinalNoTrade
                ) {
                    risk_align += 1.0;
                }
                if matches!(vote.stance, PersonaStance::NoTrade | PersonaStance::Veto) {
                    no_trade_count += 1;
                }
            }
            PersonaEvaluationMetric {
                persona_id,
                sample_count,
                stance_counts,
                avg_conviction: conviction_sum / sample_count.max(1) as f64,
                avg_voice_power: voice_sum / sample_count.max(1) as f64,
                calibration_proxy: Some(
                    (conviction_sum / sample_count.max(1) as f64).clamp(0.0, 1.0),
                ),
                no_trade_value_proxy: Some(no_trade_count as f64 / sample_count.max(1) as f64),
                risk_alignment_score: risk_align / sample_count.max(1) as f64,
                doctrine_violation_count,
                overtrade_warning: no_trade_count == 0 && sample_count >= 3,
                reason_codes: vec![ReasonCode::CommitteeEvaluationScaffoldBuilt],
            }
        })
        .collect::<Vec<_>>();
    persona_metrics.sort_by(|left, right| left.persona_id.cmp(&right.persona_id));
    let enough_samples = outcomes.len() >= 10;
    let recommendation = if !enough_samples {
        CommitteeEvaluationRecommendation::NotEnoughSamples
    } else if persona_metrics
        .iter()
        .any(|metric| metric.overtrade_warning || metric.doctrine_violation_count > 0)
    {
        CommitteeEvaluationRecommendation::ImprovePersonaThresholds
    } else {
        CommitteeEvaluationRecommendation::KeepCurrentPersonas
    };
    CommitteeEvaluationScaffold {
        chair_metrics: vec![format!("decision_count={}", outcomes.len())],
        risk_metrics: vec![format!(
            "denied_count={}",
            outcomes
                .iter()
                .filter(|outcome| {
                    matches!(
                        outcome.final_action,
                        CommitteeFinalAction::FinalDenied | CommitteeFinalAction::FinalNoTrade
                    )
                })
                .count()
        )],
        persona_metrics,
        enough_samples,
        recommendation,
        reason_codes: vec![ReasonCode::CommitteeEvaluationScaffoldBuilt],
    }
}

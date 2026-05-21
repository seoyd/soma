mod support;

use std::collections::BTreeMap;

use soma_zero::{
    MemberPromotionDemotionAction, MultiAxisMemberScorecard, MultiAxisMemberScorecardStatus,
    PromotionAxis,
};
use support::sprint98_support::run_sprint98;

#[test]
fn promotion_demotion_policy_is_multi_axis_and_sensitive_to_risk_discipline() {
    let bundle = run_sprint98(
        "soma_sprint98_committee_owned_core.toml",
        "promotion-demotion-policy",
    );
    for axis in [
        PromotionAxis::ReturnQuality,
        PromotionAxis::Calibration,
        PromotionAxis::DrawdownControl,
        PromotionAxis::RiskGovernorAlignment,
        PromotionAxis::NoTradeDiscipline,
        PromotionAxis::RiskDeniedRespect,
        PromotionAxis::EvidenceQuality,
        PromotionAxis::SourceBoundaryDiscipline,
        PromotionAxis::NoLookaheadDiscipline,
        PromotionAxis::DebateContribution,
        PromotionAxis::RegimeSpecialization,
        PromotionAxis::DefensiveValue,
        PromotionAxis::OpportunityCostAwareness,
        PromotionAxis::OverfitRisk,
    ] {
        assert!(
            bundle.promotion_demotion_policy.axes.contains(&axis),
            "missing {axis:?}"
        );
    }
    let mut strong_scores = BTreeMap::new();
    for axis in &bundle.promotion_demotion_policy.axes {
        strong_scores.insert(
            *axis,
            if *axis == PromotionAxis::OverfitRisk {
                0.2
            } else {
                0.9
            },
        );
    }
    let promote = bundle.promotion_demotion_policy.evaluate_member(
        "promote-me",
        3,
        &MultiAxisMemberScorecard {
            scorecard_id: "promote".to_string(),
            member_id: "promote-me".to_string(),
            axis_scores: strong_scores.clone(),
            recent_proposals: vec![],
            debate_turn_quality: 0.9,
            risk_alignment_score: 0.95,
            no_trade_discipline_score: 0.9,
            calibration_score: 0.9,
            overfit_risk_score: 0.2,
            overall_research_rank: 3,
            scorecard_status: MultiAxisMemberScorecardStatus::ScorecardReady,
            reason_codes: vec![],
        },
        &bundle.chairman_rulebook_version.version_id,
    );
    assert_eq!(promote.action, MemberPromotionDemotionAction::Promote);
    let mut weak_scores = strong_scores.clone();
    weak_scores.insert(PromotionAxis::RiskGovernorAlignment, 0.2);
    weak_scores.insert(PromotionAxis::NoTradeDiscipline, 0.2);
    weak_scores.insert(PromotionAxis::OverfitRisk, 0.7);
    let demote = bundle.promotion_demotion_policy.evaluate_member(
        "demote-me",
        3,
        &MultiAxisMemberScorecard {
            scorecard_id: "demote".to_string(),
            member_id: "demote-me".to_string(),
            axis_scores: weak_scores.clone(),
            recent_proposals: vec![],
            debate_turn_quality: 0.4,
            risk_alignment_score: 0.2,
            no_trade_discipline_score: 0.2,
            calibration_score: 0.4,
            overfit_risk_score: 0.7,
            overall_research_rank: 3,
            scorecard_status: MultiAxisMemberScorecardStatus::ScorecardReadyWithWarnings,
            reason_codes: vec![],
        },
        &bundle.chairman_rulebook_version.version_id,
    );
    assert!(matches!(
        demote.action,
        MemberPromotionDemotionAction::Demote | MemberPromotionDemotionAction::Watchlist
    ));
    let mut retire_scores = strong_scores;
    retire_scores.insert(PromotionAxis::OverfitRisk, 0.95);
    let retire = bundle.promotion_demotion_policy.evaluate_member(
        "retire-me",
        2,
        &MultiAxisMemberScorecard {
            scorecard_id: "retire".to_string(),
            member_id: "retire-me".to_string(),
            axis_scores: retire_scores,
            recent_proposals: vec![],
            debate_turn_quality: 0.2,
            risk_alignment_score: 0.2,
            no_trade_discipline_score: 0.2,
            calibration_score: 0.2,
            overfit_risk_score: 0.95,
            overall_research_rank: 2,
            scorecard_status: MultiAxisMemberScorecardStatus::ScorecardReadyWithWarnings,
            reason_codes: vec![],
        },
        &bundle.chairman_rulebook_version.version_id,
    );
    assert_eq!(
        retire.action,
        MemberPromotionDemotionAction::RetireToDiagnostic
    );
}

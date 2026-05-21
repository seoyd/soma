use std::collections::BTreeSet;

use soma_zero::{active_persona_cards_lite, active_trinity_scorers, all_persona_cards_lite};

#[test]
fn three_active_persona_cards_exist() {
    let cards = active_persona_cards_lite();
    let ids = cards
        .iter()
        .map(|card| card.persona_id.as_str())
        .collect::<Vec<_>>();
    assert_eq!(cards.len(), 3);
    assert_eq!(
        ids,
        vec![
            "trend_breakout_fast",
            "defensive_value_risk",
            "cycle_regime_guard"
        ]
    );
}

#[test]
fn active_personas_have_immutable_doctrine_and_bounded_policy() {
    for card in active_persona_cards_lite() {
        assert!(!card.immutable_doctrine.is_empty(), "{}", card.persona_id);
        assert!(card.mutable_policy.is_bounded(), "{}", card.persona_id);
    }
}

#[test]
fn inactive_future_personas_do_not_vote() {
    let scorer_ids = active_trinity_scorers()
        .into_iter()
        .map(|scorer| scorer.card().persona_id)
        .collect::<BTreeSet<_>>();
    let inactive_ids = all_persona_cards_lite()
        .into_iter()
        .filter(|card| !card.active)
        .map(|card| card.persona_id)
        .collect::<Vec<_>>();
    for inactive_id in inactive_ids {
        assert!(!scorer_ids.contains(&inactive_id), "{inactive_id}");
    }
}

#[test]
fn archetype_labels_are_explicitly_not_literal_reproductions() {
    for card in active_persona_cards_lite() {
        assert!(card.archetype_label.contains("archetype label"));
        assert!(
            card.archetype_label
                .contains("not a literal investor reproduction")
        );
    }
}

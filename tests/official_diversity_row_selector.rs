#[path = "support/sprint48_support.rs"]
mod support;

use soma_zero::league::DiversitySelectionReason;
use soma_zero::{
    BarrierProfileRegistryBuilder, ComparableEvidenceSourceClass, OfficialDiversityCandidateRow,
    OfficialDiversityRowSelector, OfficialDiversityRowSelectorConfig,
    OfficialDiversityRowSelectorStatus, ProviderMarket,
};

fn candidate(
    id: &str,
    symbol: &str,
    timeframe: &str,
    horizon_bars: usize,
) -> OfficialDiversityCandidateRow {
    OfficialDiversityCandidateRow {
        candidate_id: id.to_string(),
        symbol: symbol.to_string(),
        market: ProviderMarket::USEquity,
        venue: Some("NASDAQ".to_string()),
        timeframe: timeframe.to_string(),
        horizon_bars,
        timestamp_ms: 1_700_000_000_000,
        source_class: ComparableEvidenceSourceClass::OfficialNonCrypto,
        available_candle_window: true,
        preregistered_profile_id: None,
        expected_official_complete_possible: true,
        diagnostic_only: false,
        selection_reasons: vec![],
        reason_codes: vec![],
    }
}

#[test]
fn candidates_adding_new_symbol_are_prioritized() {
    let baseline = support::all_tp_set();
    let registry = BarrierProfileRegistryBuilder::default()
        .build(&support::barrier_profiles_primary(
            "selector-new-symbol-registry",
        ))
        .expect("registry");
    let config = OfficialDiversityRowSelectorConfig {
        selector_id: "selector-new-symbol".to_string(),
        max_candidates: 1,
        ..OfficialDiversityRowSelectorConfig::default()
    };

    let report = OfficialDiversityRowSelector::default().run_from_candidates(
        &config,
        Some(&baseline),
        Some(&registry),
        vec![
            candidate("same-symbol", "AAPL", "1d", 3),
            candidate("new-symbol", "NVDA", "1d", 3),
        ],
    );

    assert_eq!(
        report.selector_status,
        OfficialDiversityRowSelectorStatus::CandidatesSelected
    );
    assert_eq!(report.selected_candidates[0].candidate_id, "new-symbol");
    assert!(
        report.selected_candidates[0]
            .selection_reasons
            .contains(&DiversitySelectionReason::AddsNewSymbol)
    );
}

#[test]
fn candidates_adding_new_timeframe_are_prioritized() {
    let baseline = support::all_tp_set();
    let registry = BarrierProfileRegistryBuilder::default()
        .build(&support::barrier_profiles_primary(
            "selector-new-timeframe-registry",
        ))
        .expect("registry");
    let config = OfficialDiversityRowSelectorConfig {
        selector_id: "selector-new-timeframe".to_string(),
        max_candidates: 1,
        ..OfficialDiversityRowSelectorConfig::default()
    };

    let report = OfficialDiversityRowSelector::default().run_from_candidates(
        &config,
        Some(&baseline),
        Some(&registry),
        vec![
            candidate("same-timeframe", "AAPL", "1d", 3),
            candidate("new-timeframe", "AAPL", "4h", 3),
        ],
    );

    assert_eq!(report.selected_candidates[0].candidate_id, "new-timeframe");
    assert!(
        report.selected_candidates[0]
            .selection_reasons
            .contains(&DiversitySelectionReason::AddsNewTimeframe)
    );
}

#[test]
fn candidates_adding_new_horizon_are_prioritized() {
    let baseline = support::all_tp_set();
    let registry = BarrierProfileRegistryBuilder::default()
        .build(&support::barrier_profiles_primary(
            "selector-new-horizon-registry",
        ))
        .expect("registry");
    let config = OfficialDiversityRowSelectorConfig {
        selector_id: "selector-new-horizon".to_string(),
        max_candidates: 1,
        ..OfficialDiversityRowSelectorConfig::default()
    };

    let report = OfficialDiversityRowSelector::default().run_from_candidates(
        &config,
        Some(&baseline),
        Some(&registry),
        vec![
            candidate("same-horizon", "AAPL", "1d", 3),
            candidate("new-horizon", "AAPL", "1d", 5),
        ],
    );

    assert_eq!(report.selected_candidates[0].candidate_id, "new-horizon");
    assert!(
        report.selected_candidates[0]
            .selection_reasons
            .contains(&DiversitySelectionReason::AddsNewHorizon)
    );
}

#[test]
fn selector_does_not_peek_at_outcomes_for_official_selection() {
    let config = support::row_selector_config("selector-no-peek");
    let report = OfficialDiversityRowSelector::default()
        .run(&config)
        .expect("selector report");

    assert!(
        report
            .selected_candidates
            .iter()
            .any(|candidate| candidate.candidate_id == "nvda-te")
    );
    for candidate in report
        .selected_candidates
        .iter()
        .chain(report.skipped_candidates.iter())
    {
        assert!(
            !candidate
                .selection_reasons
                .contains(&DiversitySelectionReason::AddsStopLossCandidate)
        );
        assert!(
            !candidate
                .selection_reasons
                .contains(&DiversitySelectionReason::AddsTimeExpiredCandidate)
        );
        assert!(
            !candidate
                .selection_reasons
                .contains(&DiversitySelectionReason::ReducesOutcomeConcentration)
        );
    }
}

#[test]
fn selector_can_mark_diagnostic_only_candidates() {
    let baseline = support::all_tp_set();
    let registry = BarrierProfileRegistryBuilder::default()
        .build(&support::barrier_profiles_primary(
            "selector-diagnostic-registry",
        ))
        .expect("registry");
    let config = OfficialDiversityRowSelectorConfig {
        selector_id: "selector-diagnostic".to_string(),
        max_candidates: 1,
        ..OfficialDiversityRowSelectorConfig::default()
    };
    let mut diagnostic = candidate("diagnostic", "AAPL", "4h", 5);
    diagnostic.diagnostic_only = true;

    let report = OfficialDiversityRowSelector::default().run_from_candidates(
        &config,
        Some(&baseline),
        Some(&registry),
        vec![diagnostic],
    );

    assert!(report.selected_candidates.is_empty());
    assert_eq!(report.skipped_candidates[0].candidate_id, "diagnostic");
    assert!(
        report.skipped_candidates[0]
            .selection_reasons
            .contains(&DiversitySelectionReason::DiagnosticOnly)
    );
}

#[test]
fn budget_limited_selection_is_deterministic() {
    let config = support::row_selector_config("selector-budget");
    let mut config = config;
    config.max_candidates = 1;

    let first = OfficialDiversityRowSelector::default()
        .run(&config)
        .expect("first report");
    let second = OfficialDiversityRowSelector::default()
        .run(&config)
        .expect("second report");

    assert_eq!(first, second);
    assert_eq!(first.selected_candidates.len(), 1);
}

#[test]
fn source_ineligible_candidates_are_skipped() {
    let baseline = support::all_tp_set();
    let registry = BarrierProfileRegistryBuilder::default()
        .build(&support::barrier_profiles_primary(
            "selector-source-registry",
        ))
        .expect("registry");
    let config = OfficialDiversityRowSelectorConfig {
        selector_id: "selector-source-ineligible".to_string(),
        max_candidates: 1,
        ..OfficialDiversityRowSelectorConfig::default()
    };
    let mut candidate = candidate("crypto", "BTCUSD", "1d", 3);
    candidate.source_class = ComparableEvidenceSourceClass::OfficialCryptoOnly;
    candidate.expected_official_complete_possible = false;

    let report = OfficialDiversityRowSelector::default().run_from_candidates(
        &config,
        Some(&baseline),
        Some(&registry),
        vec![candidate],
    );

    assert!(report.selected_candidates.is_empty());
    assert!(
        report.skipped_candidates[0]
            .selection_reasons
            .contains(&DiversitySelectionReason::SourceIneligible)
    );
}

#[test]
fn selector_is_deterministic() {
    let config = support::row_selector_config("selector-deterministic");

    let first = OfficialDiversityRowSelector::default()
        .run(&config)
        .expect("first selector report");
    let second = OfficialDiversityRowSelector::default()
        .run(&config)
        .expect("second selector report");

    assert_eq!(first, second);
    assert_eq!(first.to_text(), second.to_text());
    assert_eq!(first.fingerprint(), second.fingerprint());
}

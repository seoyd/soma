use std::process::Command;

use soma_zero::league::minimal_ai_committee_core::{
    AICommitteeMember, AICommitteeMemberStatus, AIRuntimeMode, AiMemberBrain, AiMemberCoreRegistry,
    ArchetypeRiskBias, ArchetypeStyleCardRegistry, ArchetypeStyleTag, BatchCommitteeCycleInput,
    BatchCommitteeCycleWithStateInput, ChairmanFinalAction, CoreAwareMemberBrainAdapter,
    CoreRuntimeStatus, DataRouterInput, DeterministicMockBrain, EvidencePreference,
    IndependentMemberRole, InvestmentEventQueue, InvestorArchetypeStyleCard,
    Mamba3GatedDeltaNetCoreSpec, MarketScope, MemberActivationPolicy, MemberCoreFamily,
    MemberInputPacket, MemberLearningSignal, MemberScoreUpdateReason, MemberSelectionSkipReason,
    MemberStance, MemberStateStore, MemberStyleStatus, MemoryCoreKind,
    MinimalAiCommitteeCycleConfig, OfflineMemberBrainAdapter, OfflineMemberOpinionFixture,
    OfflineMemberOutputBatch, PreferredMarketBias, PreferredTimeHorizon, RealArchetypeIntakePolicy,
    RiskGovernorStatus, SequenceCoreKind, SourceConfidence, StyleCardStatus, StyleMappingMode,
    create_three_member_pilot_roster, map_style_cards_to_three_member_pilot,
    market_committee_layouts, route_data_to_ai_members, run_batch_committee_cycle_from_config_path,
    run_batch_committee_cycle_with_state, run_batch_committee_cycle_with_state_from_config_path,
    run_minimal_committee_cycle,
};

fn single_cycle_config() -> MinimalAiCommitteeCycleConfig {
    MinimalAiCommitteeCycleConfig {
        input_path: Some("examples/minimal_ai_committee_core_sample.json".to_string()),
        offline_member_opinion_path: Some(
            "examples/minimal_ai_committee_offline_member_sample.json".to_string(),
        ),
        offline_member_output_batch_path: None,
        batch_mode: false,
        member_state_input_path: None,
        member_state_output_path: None,
        emit_owner_summary: false,
        inline_offline_member_opinions: Vec::new(),
        inline_input: None,
        pilot_roster: Some("three_member".to_string()),
        paper_outcome: Some(
            soma_zero::league::minimal_ai_committee_core::SimulatedPaperOutcome::Positive,
        ),
        archetype_style_cards_path: Some(
            "examples/investor_archetype_style_cards.sample.json".to_string(),
        ),
        style_mapping_mode: StyleMappingMode::LocalFixture,
    }
}

#[test]
fn market_scope_split_exists_without_runtime_paths() {
    let scopes = [
        MarketScope::KoreaShortTerm,
        MarketScope::KoreaLongTerm,
        MarketScope::UsShortTerm,
        MarketScope::UsLongTerm,
        MarketScope::CryptoShortTerm,
        MarketScope::CryptoLongTerm,
    ];
    let text = serde_json::to_string(&scopes).expect("serialize scopes");
    for required in [
        "KoreaShortTerm",
        "KoreaLongTerm",
        "UsShortTerm",
        "UsLongTerm",
        "CryptoShortTerm",
        "CryptoLongTerm",
    ] {
        assert!(text.contains(required));
    }
}

#[test]
fn minimal_ai_committee_cycle_runs_paper_only_event_loop() {
    let result = run_minimal_committee_cycle(
        single_cycle_config()
            .load_input()
            .expect("load single input"),
    )
    .expect("minimal committee cycle runs");

    assert_eq!(result.selected_scope, MarketScope::KoreaShortTerm);
    assert_eq!(result.symbol, "005930.KS");
    assert_eq!(result.routed_packet_count, 3);
    assert_eq!(
        result.activation_plan.committee_name,
        "KoreaShortTermCommittee"
    );
    assert_eq!(result.activation_plan.selected_member_ids.len(), 3);
    assert!(result.activation_plan.estimated_memory_hint_mb <= 150);
    assert!(
        result
            .activation_plan
            .runtime_status_by_member
            .iter()
            .all(
                |status| status.runtime_status == CoreRuntimeStatus::OfflineFixture
                    || status.member_id == "crypto-liquidity"
            )
    );
    assert_eq!(result.member_roles.len(), 3);
    assert!(result.member_roles.iter().any(|role| {
        role.member_id == "trend-kr-short" && role.role == IndependentMemberRole::TrendEntry
    }));
    assert_eq!(result.memory_states.len(), 3);
    assert!(result.memory_states.iter().all(|state| {
        state.recent_symbols.contains(&"005930.KS".to_string()) && state.recent_opinion_count == 1
    }));
    assert_eq!(result.learning_journal_entry_count, 3);
    assert_eq!(result.learning_journals.len(), 3);
    let registry = result
        .style_card_registry
        .as_ref()
        .expect("style card registry");
    assert_eq!(registry.cards.len(), 18);
    assert!(registry.review_required_count >= 3);
    let style_mapping = result
        .three_member_style_mapping
        .as_ref()
        .expect("three-member style mapping");
    assert_eq!(style_mapping.trend_entry_blend.member_id, "trend-kr-short");
    assert_eq!(result.style_influenced_profiles.len(), 3);
    assert!(result.style_influenced_profiles.iter().any(|profile| {
        profile.member_id == "risk-kr-short"
            && profile
                .decision_bias_notes
                .iter()
                .any(|note| note.contains("Risk Governor"))
    }));
    let journal_member_ids: std::collections::BTreeSet<_> = result
        .learning_journals
        .iter()
        .map(|journal| journal.member_id.as_str())
        .collect();
    assert_eq!(journal_member_ids.len(), 3);
    assert!(result.learning_journal_entries.iter().any(|entry| {
        entry.member_id == "risk-kr-short"
            && entry.learning_signal == MemberLearningSignal::Reinforce
    }));
    assert_eq!(result.triggered_event_count, 1);
    assert_eq!(result.committee_session_count, 1);
    assert_eq!(
        result.distributed_member_ids,
        vec![
            "trend-kr-short".to_string(),
            "risk-kr-short".to_string(),
            "evidence-kr-short".to_string()
        ]
    );
    assert!(
        !result
            .distributed_member_ids
            .contains(&"crypto-liquidity".to_string())
    );
    assert_eq!(result.member_opinions.len(), 3);
    assert!(result.member_opinions.iter().all(|opinion| {
        opinion.symbol == "005930.KS" && opinion.market_scope == MarketScope::KoreaShortTerm
    }));
    let trend_opinion = result
        .member_opinions
        .iter()
        .find(|opinion| opinion.member_id == "trend-kr-short")
        .expect("trend opinion");
    assert_eq!(trend_opinion.stance, MemberStance::BuyProposal);
    assert!(trend_opinion.event_triggered);
    assert!(
        trend_opinion
            .evidence_notes
            .contains(&"offline fixture opinion".to_string())
    );
    assert_ne!(
        trend_opinion.evidence_notes,
        result
            .member_opinions
            .iter()
            .find(|opinion| opinion.member_id == "risk-kr-short")
            .expect("risk opinion")
            .evidence_notes
    );
    let risk_opinion = result
        .member_opinions
        .iter()
        .find(|opinion| opinion.member_id == "risk-kr-short")
        .expect("risk opinion");
    assert_eq!(risk_opinion.stance, MemberStance::NoTrade);
    assert!(risk_opinion.event_triggered);
    let evidence_opinion = result
        .member_opinions
        .iter()
        .find(|opinion| opinion.member_id == "evidence-kr-short")
        .expect("evidence opinion");
    assert_eq!(evidence_opinion.stance, MemberStance::Hold);
    assert!(!evidence_opinion.event_triggered);

    let event = result.event.as_ref().expect("investment event");
    assert_eq!(event.proposed_by_member_id, "trend-kr-short");
    let session = result
        .committee_session
        .as_ref()
        .expect("committee session");
    assert_eq!(session.invited_members.len(), 3);
    assert!(session.risk_flags.contains(&"high_volatility".to_string()));

    let decision = result
        .chairman_decision
        .as_ref()
        .expect("chairman decision");
    assert_eq!(decision.final_action, ChairmanFinalAction::RiskVetoed);
    assert_eq!(decision.risk_governor_status, RiskGovernorStatus::Vetoed);
    assert!(decision.rationale.contains("Risk Governor vetoed"));
    assert!(result.learning_journal_entries.iter().all(|entry| {
        entry.journal_id.contains(&decision.decision_id) && entry.note.contains("no model training")
    }));

    let risk_update = result
        .score_updates
        .iter()
        .find(|update| update.member_id == "risk-kr-short")
        .expect("risk score update");
    assert_eq!(
        risk_update.update_reason,
        MemberScoreUpdateReason::HelpfulDissent
    );
    assert!(risk_update.new_voice_weight > risk_update.previous_voice_weight);
    let trend_update = result
        .score_updates
        .iter()
        .find(|update| update.member_id == "trend-kr-short")
        .expect("trend score update");
    assert_eq!(
        trend_update.update_reason,
        MemberScoreUpdateReason::RiskyCall
    );
    assert!(trend_update.new_voice_weight < trend_update.previous_voice_weight);

    assert!(result.safety_summary.paper_only);
    assert!(result.safety_summary.no_real_order_path);
    assert!(result.safety_summary.no_broker_order_account);
    assert!(result.safety_summary.no_model_training);
    assert!(result.safety_summary.no_live_inference);
}

#[test]
fn minimal_ai_committee_cycle_is_deterministic_and_cli_is_safe() {
    let first = run_minimal_committee_cycle(
        single_cycle_config()
            .load_input()
            .expect("load first input"),
    )
    .expect("first run");
    let second = run_minimal_committee_cycle(
        single_cycle_config()
            .load_input()
            .expect("load second input"),
    )
    .expect("second run");
    assert_eq!(first, second);

    let binary = env!("CARGO_BIN_EXE_soma_experiment");
    let output = Command::new(binary)
        .args([
            "minimal-ai-committee-cycle",
            "--config",
            "examples/soma_minimal_ai_committee_core.toml",
        ])
        .output()
        .expect("run minimal committee CLI");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("minimal_ai_committee_warning"));
    assert!(stdout.contains("paper-only local mock/offline fixture member logic"));
    assert!(stdout.contains("\"batch_id\": \"offline-batch-sprint130-sample\""));
    assert!(stdout.contains("\"routed_packet_count\": 18"));
    assert!(stdout.contains("\"member_opinion_count\": 18"));
    assert!(stdout.contains("\"event_count\": 5"));
    assert!(stdout.contains("\"events_by_symbol\""));
    assert!(stdout.contains("\"risk_veto_count\""));
    assert!(stdout.contains("\"score_update_count\""));
    assert!(stdout.contains("\"owner_summary\""));
    assert!(stdout.contains("\"member_voice_changes\""));
    assert!(stdout.contains("paper-only explanation"));
    assert!(stdout.contains("offline batch opinion"));
    assert!(stdout.contains("\"no_broker_order_account\": true"));
    assert!(stdout.contains("\"final_action\": \"RiskVetoed\""));

    let help = Command::new(binary)
        .args(["minimal-ai-committee-cycle", "--help"])
        .output()
        .expect("run minimal committee help");
    assert!(help.status.success());
    let help_text = String::from_utf8_lossy(&help.stdout);
    assert!(help_text.contains("no broker/order/account"));
    assert!(help_text.contains("no training"));
    assert!(help_text.contains("no live inference"));

    let remote = Command::new(binary)
        .args([
            "minimal-ai-committee-cycle",
            "--config",
            "https://example.invalid/config.toml",
        ])
        .output()
        .expect("run remote config rejection");
    assert!(!remote.status.success());
    assert!(String::from_utf8_lossy(&remote.stderr).contains("config path must be local"));
}

#[test]
fn offline_member_brain_adapter_returns_fixture_and_falls_back_safely() {
    let fixtures = vec![OfflineMemberOpinionFixture {
        member_id: "offline-member".to_string(),
        symbol: "005930.KS".to_string(),
        market_scope: MarketScope::KoreaShortTerm,
        stance: MemberStance::BuyProposal,
        confidence: 0.77,
        expected_return_hint: 0.02,
        risk_hint: 0.03,
        evidence_notes: vec![
            "offline fixture opinion".to_string(),
            "local file only".to_string(),
        ],
        event_triggered: true,
        event_reason: Some("fixture event".to_string()),
    }];
    let adapter = OfflineMemberBrainAdapter { fixtures };
    let packet = MemberInputPacket {
        member_id: "offline-member".to_string(),
        market_data: serde_json::from_value(serde_json::json!({
            "symbol": "005930.KS",
            "market_scope": "KoreaShortTerm",
            "timestamp": "2026-05-21T09:00:00+09:00",
            "price": 78000.0,
            "change_pct": 2.0,
            "volume": 1000.0,
            "volatility_hint": 0.03,
            "source_label": "test"
        }))
        .expect("market data"),
        news: Vec::new(),
        owner_context: Some("paper-only".to_string()),
        previous_member_score: Some(0.6),
    };
    let opinion = adapter.produce_opinion(&packet);
    assert_eq!(opinion.stance, MemberStance::BuyProposal);
    assert!(opinion.event_triggered);
    assert!(
        opinion
            .evidence_notes
            .contains(&"local file only".to_string())
    );

    let missing_packet = MemberInputPacket {
        member_id: "missing-member".to_string(),
        ..packet
    };
    let fallback = adapter.produce_opinion(&missing_packet);
    assert_eq!(fallback.stance, MemberStance::NeedMoreEvidence);
    assert!(!fallback.event_triggered);
    assert!(
        fallback
            .evidence_notes
            .iter()
            .any(|note| note.contains("no external model"))
    );
}

#[test]
fn offline_fixture_path_is_local_only() {
    let err = OfflineMemberBrainAdapter::from_json_path(std::path::Path::new(
        "https://example.invalid/offline.json",
    ))
    .expect_err("remote offline fixture must fail");
    assert!(err.contains("must be local"));

    let config = MinimalAiCommitteeCycleConfig {
        input_path: Some("examples/minimal_ai_committee_core_sample.json".to_string()),
        offline_member_opinion_path: Some("https://example.invalid/offline.json".to_string()),
        offline_member_output_batch_path: None,
        batch_mode: false,
        member_state_input_path: None,
        member_state_output_path: None,
        emit_owner_summary: false,
        inline_offline_member_opinions: Vec::new(),
        inline_input: None,
        pilot_roster: None,
        paper_outcome: None,
        archetype_style_cards_path: None,
        style_mapping_mode: StyleMappingMode::None,
    };
    let err = config
        .validate()
        .expect_err("remote offline fixture config must fail");
    assert!(err.contains("offline_member_opinion_path must be local"));
}

#[test]
fn offline_member_output_batch_loads_rejects_unsafe_content_and_matches_packets() {
    let load = OfflineMemberOutputBatch::from_json_path(std::path::Path::new(
        "examples/minimal_offline_member_output_batch.sample.json",
    ))
    .expect("load offline output batch");
    assert_eq!(load.batch_id, "offline-batch-sprint130-sample");
    assert_eq!(load.loaded_count, 5);
    assert_eq!(load.invalid_count, 0);
    assert_eq!(load.duplicate_count, 0);
    assert_eq!(load.unmatched_count, 0);
    assert!(
        load.safety_notes
            .iter()
            .any(|note| note.contains("no network"))
    );

    let err = OfflineMemberOutputBatch::from_json_path(std::path::Path::new(
        "https://example.invalid/batch.json",
    ))
    .expect_err("remote batch path must fail");
    assert!(err.contains("must be local"));

    let unsafe_json = serde_json::json!({
        "batch_id": "unsafe",
        "created_at": "2026-05-22T17:43:38+09:00",
        "source_label": "local-test",
        "broker": "not allowed",
        "opinions": []
    })
    .to_string();
    let err =
        OfflineMemberOutputBatch::from_json_str(&unsafe_json).expect_err("broker field must fail");
    assert!(err.contains("unsafe field"));

    let unsafe_key_json = serde_json::json!({
        "batch_id": "unsafe-key",
        "created_at": "2026-05-22T17:43:38+09:00",
        "source_label": "local-test",
        "order_instruction": "not allowed",
        "opinions": []
    })
    .to_string();
    let err = OfflineMemberOutputBatch::from_json_str(&unsafe_key_json)
        .expect_err("order instruction field must fail");
    assert!(err.contains("unsafe field"));

    let unsafe_claim_json = serde_json::json!({
        "batch_id": "unsafe-claim",
        "created_at": "2026-05-22T17:43:38+09:00",
        "source_label": "local-test",
        "opinions": [{
            "member_id": "trend-kr-short",
            "symbol": "005930.KS",
            "market_scope": "KoreaShortTerm",
            "stance": "BuyProposal",
            "confidence": 0.7,
            "expected_return_hint": 0.02,
            "risk_hint": 0.03,
            "evidence_notes": ["guaranteed returns from a private strategy"],
            "event_triggered": true,
            "event_reason": "unsafe"
        }]
    })
    .to_string();
    let err = OfflineMemberOutputBatch::from_json_str(&unsafe_claim_json)
        .expect_err("guaranteed return/private strategy claim must fail");
    assert!(err.contains("unsafe claim"));

    let unsafe_impersonation_json = serde_json::json!({
        "batch_id": "unsafe-impersonation",
        "created_at": "2026-05-22T17:43:38+09:00",
        "source_label": "local-test",
        "opinions": [{
            "member_id": "trend-kr-short",
            "symbol": "005930.KS",
            "market_scope": "KoreaShortTerm",
            "stance": "BuyProposal",
            "confidence": 0.7,
            "expected_return_hint": 0.02,
            "risk_hint": 0.03,
            "evidence_notes": ["trades like Warren Buffett AI"],
            "event_triggered": true,
            "event_reason": "unsafe"
        }]
    })
    .to_string();
    let err = OfflineMemberOutputBatch::from_json_str(&unsafe_impersonation_json)
        .expect_err("impersonation claim must fail");
    assert!(err.contains("unsafe claim"));

    let adapter = OfflineMemberBrainAdapter {
        fixtures: load.opinions,
    };
    let packet = MemberInputPacket {
        member_id: "risk-kr-short".to_string(),
        market_data: serde_json::from_value(serde_json::json!({
            "symbol": "BTCUSDT",
            "market_scope": "CryptoShortTerm",
            "timestamp": "2026-05-21T00:00:00Z",
            "price": 108000.0,
            "change_pct": 4.8,
            "volume": 4100000000.0,
            "volatility_hint": 0.09,
            "source_label": "local-sample"
        }))
        .expect("market data"),
        news: Vec::new(),
        owner_context: Some("paper-only".to_string()),
        previous_member_score: Some(0.64),
    };
    let opinion = adapter.produce_opinion(&packet);
    assert_eq!(opinion.member_id, "risk-kr-short");
    assert_eq!(opinion.symbol, "BTCUSDT");
    assert_eq!(opinion.market_scope, MarketScope::CryptoShortTerm);
    assert_eq!(opinion.stance, MemberStance::NoTrade);
}

#[test]
fn batch_committee_cycle_routes_multi_symbol_events_and_preserves_safety() {
    let sample = std::fs::read_to_string("examples/minimal_ai_committee_multi_market_sample.json")
        .expect("batch input sample");
    let batch_input: BatchCommitteeCycleInput =
        serde_json::from_str(&sample).expect("parse batch input sample");
    assert_eq!(batch_input.market_data.len(), 6);

    let result = run_batch_committee_cycle_from_config_path(std::path::Path::new(
        "examples/soma_minimal_ai_committee_core.toml",
    ))
    .expect("batch cycle runs");
    assert_eq!(result.batch_id, "offline-batch-sprint130-sample");
    assert_eq!(result.routed_packet_count, 18);
    assert_eq!(result.member_opinion_count, 18);
    assert_eq!(result.event_queue.event_count, 5);
    assert_eq!(
        result.committee_sessions.len(),
        result.event_queue.event_count
    );
    assert_eq!(
        result.chairman_decisions.len(),
        result.event_queue.event_count
    );
    assert!(result.risk_veto_count >= 1);
    assert_eq!(result.score_update_count, result.score_updates.len());
    assert_eq!(
        result.learning_journal_entry_count,
        result.learning_journal_entries.len()
    );
    assert!(result.learning_journal_entry_count >= result.score_update_count);
    assert!(result.events_by_symbol.contains_key("005930.KS"));
    assert!(result.events_by_symbol.contains_key("BTCUSDT"));
    assert!(result.event_queue.symbols.contains(&"AAPL".to_string()));
    assert!(
        result
            .event_queue
            .market_scopes
            .contains(&MarketScope::CryptoShortTerm)
    );
    assert_eq!(
        result
            .event_queue
            .events
            .first()
            .expect("first event")
            .event_type,
        soma_zero::league::minimal_ai_committee_core::InvestmentEventType::RiskWarning
    );
    let highest = result
        .event_queue
        .highest_confidence_event()
        .expect("highest confidence event");
    assert!(highest.triggering_opinion.confidence >= 0.8);

    let missing_fallback = result
        .member_opinions
        .iter()
        .find(|opinion| opinion.symbol == "MSFT" && opinion.member_id == "trend-kr-short")
        .expect("missing offline opinion fallback");
    assert_eq!(missing_fallback.stance, MemberStance::NeedMoreEvidence);
    assert!(!missing_fallback.event_triggered);
    assert!(
        missing_fallback
            .evidence_notes
            .iter()
            .any(|note| note.contains("offline fixture missing"))
    );

    assert!(result.safety_summary.paper_only);
    assert!(result.safety_summary.no_real_order_path);
    assert!(result.safety_summary.no_broker_order_account);
    assert!(result.safety_summary.no_model_training);
    assert!(result.safety_summary.no_live_inference);

    let second = run_batch_committee_cycle_from_config_path(std::path::Path::new(
        "examples/soma_minimal_ai_committee_core.toml",
    ))
    .expect("second batch cycle");
    assert_eq!(result, second);
}

#[test]
fn member_state_store_accumulates_batch_memory_and_persists_locally() {
    let roster = create_three_member_pilot_roster(MarketScope::KoreaShortTerm);
    let mut store = MemberStateStore::from_members("test-member-state-store", &roster, "unit-test");
    assert!(store.paper_only);
    assert_eq!(store.members.len(), 3);
    assert!(store.get_member_state("trend-kr-short").is_some());

    let err = MemberStateStore::load_from_local_json(std::path::Path::new(
        "https://example.invalid/member-state.json",
    ))
    .expect_err("remote state input path must fail");
    assert!(err.contains("must be local"));

    let batch_input = MinimalAiCommitteeCycleConfig::from_toml_path(std::path::Path::new(
        "examples/soma_minimal_ai_committee_core.toml",
    ))
    .expect("config")
    .load_batch_input()
    .expect("batch input");
    let result = run_batch_committee_cycle_with_state(BatchCommitteeCycleWithStateInput {
        batch_input,
        member_state_store: Some(store.clone()),
        member_state_output_path: None,
        emit_owner_summary: true,
    })
    .expect("stateful batch cycle");

    store.apply_score_updates(&result.batch_result.score_updates);
    store.apply_memory_updates(&result.batch_result.memory_updates);
    store.append_learning_journal_entries(&result.batch_result.learning_journal_entries);
    let risk_state = store
        .get_member_state("risk-kr-short")
        .expect("risk member state");
    assert!(
        result
            .batch_result
            .score_updates
            .iter()
            .any(|update| update.new_voice_weight != update.previous_voice_weight)
    );
    for state in &store.members {
        if let Some(update) = result
            .batch_result
            .score_updates
            .iter()
            .rev()
            .find(|update| update.member_id == state.member_id)
        {
            assert_eq!(state.score, update.new_score);
            assert_eq!(state.voice_weight, update.new_voice_weight);
        }
    }
    assert!(risk_state.memory_state.recent_opinion_count > 0);
    assert!(risk_state.memory_state.recent_event_count > 0);
    assert!(
        risk_state.learning_journal_summary.reinforce_count
            + risk_state.learning_journal_summary.penalize_count
            + risk_state.learning_journal_summary.watch_count
            + risk_state.learning_journal_summary.ignore_count
            > 0
    );

    let path = std::path::Path::new("target/sprint132_member_state_store_test.json");
    let _ = std::fs::remove_file(path);
    store
        .save_to_local_json(path)
        .expect("save local member state");
    let loaded = MemberStateStore::load_from_local_json(path).expect("reload local member state");
    assert_eq!(loaded, store);
    let _ = std::fs::remove_file(path);

    let unsafe_path = std::path::Path::new("target/sprint132_unsafe_member_state_store_test.json");
    let unsafe_state = serde_json::json!({
        "store_id": "unsafe-member-state-store",
        "members": [],
        "source_label": "unit-test",
        "paper_only": true,
        "broker_account": "not allowed"
    })
    .to_string();
    std::fs::write(unsafe_path, unsafe_state).expect("write unsafe state fixture");
    let err = MemberStateStore::load_from_local_json(unsafe_path)
        .expect_err("unsafe broker/account state field must fail");
    assert!(err.contains("unsafe field"));
    let _ = std::fs::remove_file(unsafe_path);

    let err = store
        .save_to_local_json(std::path::Path::new(
            "https://example.invalid/member-state.json",
        ))
        .expect_err("remote state output path must fail");
    assert!(err.contains("must be local"));
}

#[test]
fn batch_cycle_with_state_produces_owner_summary_and_is_deterministic() {
    let first = run_batch_committee_cycle_with_state_from_config_path(std::path::Path::new(
        "examples/soma_minimal_ai_committee_core.toml",
    ))
    .expect("first stateful batch cycle");
    let second = run_batch_committee_cycle_with_state_from_config_path(std::path::Path::new(
        "examples/soma_minimal_ai_committee_core.toml",
    ))
    .expect("second stateful batch cycle");
    assert_eq!(first, second);

    assert_eq!(
        first.state_update.cycle_id,
        "offline-batch-sprint130-sample"
    );
    assert_eq!(
        first.state_update.score_updates.len(),
        first.batch_result.score_update_count
    );
    assert!(!first.state_update.memory_updates.is_empty());
    assert_eq!(first.state_update.updated_member_states.len(), 3);
    let summary = first.owner_summary.as_ref().expect("owner summary");
    assert!(summary.symbols_reviewed.contains(&"005930.KS".to_string()));
    assert!(summary.symbols_reviewed.contains(&"BTCUSDT".to_string()));
    assert_eq!(
        summary.event_count,
        first.batch_result.event_queue.event_count
    );
    assert_eq!(summary.risk_veto_count, first.batch_result.risk_veto_count);
    assert!(!summary.member_voice_changes.is_empty());
    assert!(!summary.chairman_actions.is_empty());
    assert!(
        summary
            .risk_warnings
            .iter()
            .any(|warning| warning.contains("high_volatility"))
    );
    assert!(summary.owner_readable_summary.contains("검토 종목"));
    assert!(summary.paper_only_warning.contains("not an order"));
    assert!(first.batch_result.safety_summary.no_broker_order_account);
    assert!(first.batch_result.safety_summary.no_model_training);
    assert!(first.batch_result.safety_summary.no_live_inference);
}

#[test]
fn event_queue_groups_and_risk_first_ordering_are_deterministic() {
    let opinions = vec![
        OfflineMemberOpinionFixture {
            member_id: "trend-kr-short".to_string(),
            symbol: "005930.KS".to_string(),
            market_scope: MarketScope::KoreaShortTerm,
            stance: MemberStance::BuyProposal,
            confidence: 0.91,
            expected_return_hint: 0.03,
            risk_hint: 0.04,
            evidence_notes: vec!["paper-only event queue test".to_string()],
            event_triggered: true,
            event_reason: Some("entry".to_string()),
        },
        OfflineMemberOpinionFixture {
            member_id: "risk-kr-short".to_string(),
            symbol: "005930.KS".to_string(),
            market_scope: MarketScope::KoreaShortTerm,
            stance: MemberStance::NoTrade,
            confidence: 0.72,
            expected_return_hint: 0.0,
            risk_hint: 0.3,
            evidence_notes: vec!["paper-only event queue test".to_string()],
            event_triggered: true,
            event_reason: Some("risk".to_string()),
        },
    ]
    .into_iter()
    .map(|fixture| OfflineMemberBrainAdapter {
        fixtures: vec![fixture],
    })
    .map(|adapter| {
        let fixture = adapter.fixtures.first().expect("fixture");
        soma_zero::league::minimal_ai_committee_core::MemberOpinion {
            member_id: fixture.member_id.clone(),
            symbol: fixture.symbol.clone(),
            market_scope: fixture.market_scope,
            stance: fixture.stance,
            confidence: fixture.confidence,
            expected_return_hint: fixture.expected_return_hint,
            risk_hint: fixture.risk_hint,
            evidence_notes: fixture.evidence_notes.clone(),
            event_triggered: fixture.event_triggered,
            event_reason: fixture.event_reason.clone(),
        }
    })
    .collect::<Vec<_>>();
    let queue = InvestmentEventQueue::from_member_opinions(&opinions);
    assert_eq!(queue.event_count, 2);
    assert_eq!(
        queue.events.first().expect("risk first").event_type,
        soma_zero::league::minimal_ai_committee_core::InvestmentEventType::RiskWarning
    );
    assert_eq!(
        queue
            .group_by_symbol()
            .get("005930.KS")
            .expect("symbol group")
            .len(),
        2
    );
    assert_eq!(
        queue
            .group_by_market_scope()
            .get(&MarketScope::KoreaShortTerm)
            .expect("scope group")
            .len(),
        2
    );
}

#[test]
fn three_member_pilot_roster_has_independent_roles_memory_and_core_specs() {
    let roster = create_three_member_pilot_roster(MarketScope::KoreaShortTerm);
    assert_eq!(roster.len(), 3);
    assert_eq!(
        roster
            .iter()
            .filter_map(|member| member.role)
            .collect::<Vec<_>>(),
        vec![
            IndependentMemberRole::TrendEntry,
            IndependentMemberRole::RiskGuard,
            IndependentMemberRole::EvidenceRegime
        ]
    );
    for member in roster {
        assert_eq!(member.market_scopes, vec![MarketScope::KoreaShortTerm]);
        let core_spec = member.core_spec.as_ref().expect("core spec");
        assert_eq!(core_spec.core_family, MemberCoreFamily::Mamba3GatedDeltaNet);
        assert_eq!(core_spec.sequence_core, SequenceCoreKind::Mamba3Deferred);
        assert_eq!(core_spec.memory_core, MemoryCoreKind::GatedDeltaNetDeferred);
        let memory = member.memory_state.expect("memory state");
        assert_eq!(memory.member_id, member.member_id);
        assert_eq!(memory.recent_opinion_count, 0);
        assert!(
            memory
                .notes
                .iter()
                .any(|note| note.contains("no model weight update"))
        );
    }

    let roster = create_three_member_pilot_roster(MarketScope::KoreaShortTerm);
    let output = route_data_to_ai_members(DataRouterInput {
        market_data: vec![
            serde_json::from_value(serde_json::json!({
                "symbol": "005930.KS",
                "market_scope": "KoreaShortTerm",
                "timestamp": "2026-05-21T09:00:00+09:00",
                "price": 78000.0,
                "change_pct": 1.0,
                "volume": 1000.0,
                "volatility_hint": 0.04,
                "source_label": "test"
            }))
            .expect("market data"),
        ],
        news: Vec::new(),
        members: roster.clone(),
        owner_context: Some("paper-only".to_string()),
    });
    assert_eq!(output.packets.len(), 3);

    let evidence_member = roster
        .iter()
        .find(|member| member.role == Some(IndependentMemberRole::EvidenceRegime))
        .expect("evidence member")
        .clone();
    let evidence_packet = MemberInputPacket {
        member_id: evidence_member.member_id.clone(),
        market_data: serde_json::from_value(serde_json::json!({
            "symbol": "005930.KS",
            "market_scope": "KoreaShortTerm",
            "timestamp": "2026-05-21T09:00:00+09:00",
            "price": 78000.0,
            "change_pct": 1.0,
            "volume": 1000.0,
            "volatility_hint": 0.04,
            "source_label": "test"
        }))
        .expect("market data"),
        news: vec![
            serde_json::from_value(serde_json::json!({
                "symbol": "005930.KS",
                "headline": "unclear evidence",
                "summary": "unknown regime evidence",
                "sentiment_hint": "unknown",
                "source_label": "test",
                "timestamp": "2026-05-21T09:00:00+09:00"
            }))
            .expect("news"),
        ],
        owner_context: Some("paper-only".to_string()),
        previous_member_score: Some(evidence_member.score),
    };
    let evidence_opinion = DeterministicMockBrain {
        member: evidence_member,
    }
    .produce_opinion(&evidence_packet);
    assert_eq!(evidence_opinion.stance, MemberStance::NeedMoreEvidence);
    assert!(evidence_opinion.event_triggered);
}

#[test]
fn archetype_style_cards_load_validate_and_map_to_three_members() {
    let registry = ArchetypeStyleCardRegistry::load_style_cards_from_local_fixture(
        std::path::Path::new("examples/investor_archetype_style_cards.sample.json"),
    )
    .expect("load style cards");
    assert_eq!(registry.cards.len(), 18);
    assert!(registry.active_count >= 15);
    assert!(registry.review_required_count >= 3);
    registry
        .validate_no_impersonation()
        .expect("no impersonation wording");
    registry
        .validate_do_not_learn_guards()
        .expect("do-not-learn guards");
    assert!(
        registry
            .cards_for_role(IndependentMemberRole::TrendEntry)
            .iter()
            .any(|card| card
                .primary_style_tags
                .iter()
                .any(|tag| format!("{:?}", tag) == "Momentum"))
    );
    assert!(
        registry
            .cards_for_role(IndependentMemberRole::RiskGuard)
            .iter()
            .any(|card| card.risk_bias
                == soma_zero::league::minimal_ai_committee_core::ArchetypeRiskBias::Conservative)
    );
    assert!(
        registry
            .cards_for_role(IndependentMemberRole::EvidenceRegime)
            .iter()
            .any(|card| format!("{:?}", card.evidence_preference) == "Fundamentals")
    );

    let mapping = map_style_cards_to_three_member_pilot(&registry);
    for blend in [
        &mapping.trend_entry_blend,
        &mapping.risk_guard_blend,
        &mapping.evidence_regime_blend,
    ] {
        let weight_sum: f64 = blend
            .archetype_weights
            .iter()
            .map(|weight| weight.weight)
            .sum();
        assert!((weight_sum - 1.0).abs() < 0.0001);
        assert!(
            blend
                .prohibited_claims
                .contains(&"not a real person clone".to_string())
        );
    }
    assert!(
        mapping
            .review_required_archetypes
            .contains(&"archetype-16".to_string())
    );
    assert_eq!(
        mapping.trend_entry_blend.source_confidence_minimum,
        SourceConfidence::ReviewRequired
    );
    assert_eq!(
        mapping.trend_entry_blend.style_status,
        MemberStyleStatus::ReadyWithWarnings
    );
}

#[test]
fn review_required_style_cards_do_not_silently_upgrade_confidence() {
    let guard = vec![
        "private_life_details".to_string(),
        "exact_personality_clone".to_string(),
        "unverified_profit_claims".to_string(),
        "unsourced_quotes".to_string(),
        "illegal_or_private_info".to_string(),
    ];
    let cards = vec![
        InvestorArchetypeStyleCard {
            archetype_id: "safe-high".to_string(),
            display_name: "Safe high confidence public trend card".to_string(),
            public_style_summary:
                "Inspired by public investment philosophy; style influence only; not a real person clone."
                    .to_string(),
            preferred_time_horizon: PreferredTimeHorizon::ShortTerm,
            preferred_market_bias: PreferredMarketBias::Any,
            primary_style_tags: vec![ArchetypeStyleTag::Trend],
            risk_bias: ArchetypeRiskBias::Balanced,
            evidence_preference: EvidencePreference::PriceAction,
            do_not_learn: guard.clone(),
            source_confidence: SourceConfidence::High,
            status: StyleCardStatus::ActiveStyleCard,
        },
        InvestorArchetypeStyleCard {
            archetype_id: "review-required".to_string(),
            display_name: "Review required public momentum card".to_string(),
            public_style_summary:
                "Inspired by public investment philosophy; style influence only; not a real person clone."
                    .to_string(),
            preferred_time_horizon: PreferredTimeHorizon::ShortTerm,
            preferred_market_bias: PreferredMarketBias::Any,
            primary_style_tags: vec![ArchetypeStyleTag::Momentum],
            risk_bias: ArchetypeRiskBias::Aggressive,
            evidence_preference: EvidencePreference::PriceAction,
            do_not_learn: guard,
            source_confidence: SourceConfidence::ReviewRequired,
            status: StyleCardStatus::ReviewRequired,
        },
    ];
    let registry = ArchetypeStyleCardRegistry::from_cards(cards);
    let mapping = map_style_cards_to_three_member_pilot(&registry);
    let weights = &mapping.trend_entry_blend.archetype_weights;
    let review_weight = weights
        .iter()
        .find(|weight| weight.archetype_id == "review-required")
        .expect("review card participates with limited weight");

    assert_eq!(
        mapping.trend_entry_blend.source_confidence_minimum,
        SourceConfidence::ReviewRequired
    );
    assert_eq!(
        mapping.trend_entry_blend.style_status,
        MemberStyleStatus::ReadyWithWarnings
    );
    assert!(review_weight.weight < 0.2);
    assert!((weights.iter().map(|weight| weight.weight).sum::<f64>() - 1.0).abs() < 0.0001);
}

#[test]
fn archetype_style_card_rejects_impersonation_wording() {
    let text = std::fs::read_to_string("examples/investor_archetype_style_cards.sample.json")
        .expect("style fixture");
    let mut cards: Vec<InvestorArchetypeStyleCard> =
        serde_json::from_str(&text).expect("style cards");
    cards[0].display_name = "Warren Buffett AI".to_string();
    let registry = ArchetypeStyleCardRegistry::from_cards(cards);
    let err = registry
        .validate_no_impersonation()
        .expect_err("impersonation wording should fail");
    assert!(err.contains("impersonation"));
}

#[test]
fn real_archetype_intake_rejects_guaranteed_returns_and_preserves_review_status() {
    let text = std::fs::read_to_string("examples/investor_archetype_style_cards.sample.json")
        .expect("style fixture");
    let mut cards: Vec<InvestorArchetypeStyleCard> =
        serde_json::from_str(&text).expect("style cards");
    cards[0].public_style_summary =
        "Inspired by public investment philosophy; guaranteed return; not a real person clone."
            .to_string();
    let registry = ArchetypeStyleCardRegistry::from_cards(cards);
    let policy = RealArchetypeIntakePolicy::default();
    let err = policy
        .validate_registry(&registry)
        .expect_err("guaranteed return claim should fail");
    assert!(err.contains("guaranteed-return"));

    let text = std::fs::read_to_string("examples/investor_archetype_style_cards.sample.json")
        .expect("style fixture");
    let mut cards: Vec<InvestorArchetypeStyleCard> =
        serde_json::from_str(&text).expect("style cards");
    cards[0].public_style_summary =
        "Inspired by public investment philosophy; uses private strategy; not a real person clone."
            .to_string();
    let registry = ArchetypeStyleCardRegistry::from_cards(cards);
    let err = policy
        .validate_registry(&registry)
        .expect_err("private strategy claim should fail");
    assert!(err.contains("private-strategy"));

    let text = std::fs::read_to_string("examples/investor_archetype_style_cards.sample.json")
        .expect("style fixture");
    let mut cards: Vec<InvestorArchetypeStyleCard> =
        serde_json::from_str(&text).expect("style cards");
    let review_card = cards
        .iter_mut()
        .find(|card| card.source_confidence == SourceConfidence::ReviewRequired)
        .expect("review card");
    review_card.status = StyleCardStatus::ActiveStyleCard;
    let registry = ArchetypeStyleCardRegistry::from_cards(cards);
    let err = policy
        .validate_registry(&registry)
        .expect_err("review-required card should stay review-required");
    assert!(err.contains("ReviewRequired status"));
}

#[test]
fn real_archetype_intake_uses_local_json_only() {
    let policy = RealArchetypeIntakePolicy::default();
    let registry = policy
        .load_registry_from_local_json(std::path::Path::new(
            "examples/investor_archetype_style_cards.sample.json",
        ))
        .expect("local style fixture");
    assert_eq!(registry.cards.len(), 18);

    let err = policy
        .load_registry_from_local_json(std::path::Path::new("https://example.invalid/cards.json"))
        .expect_err("remote style cards must fail");
    assert!(err.contains("local JSON"));
}

#[test]
fn member_core_registry_limits_18_members_and_keeps_risk_member() {
    let members: Vec<_> = (0..18)
        .map(|index| {
            let member_id = if index == 17 {
                "risk-member".to_string()
            } else {
                format!("trend-member-{index:02}")
            };
            AICommitteeMember {
                member_id: member_id.clone(),
                display_name: member_id.clone(),
                market_scopes: vec![MarketScope::KoreaShortTerm],
                style_profile: if index == 17 {
                    "risk".to_string()
                } else {
                    "trend".to_string()
                },
                voice_weight: if index == 17 {
                    0.1
                } else {
                    0.95 - index as f64 * 0.02
                },
                score: 0.6,
                status: AICommitteeMemberStatus::Active,
                runtime_mode: AIRuntimeMode::MockLocal,
                core_spec: Some(Mamba3GatedDeltaNetCoreSpec::mock_local_for(&member_id)),
                role: None,
                memory_state: None,
            }
        })
        .collect();
    assert!(members.iter().all(|member| {
        member
            .core_spec
            .as_ref()
            .is_some_and(|spec| spec.runtime_status == CoreRuntimeStatus::MockLocal)
    }));

    let registry = AiMemberCoreRegistry::from_members(&members);
    let policy = MemberActivationPolicy::default();
    let plan =
        registry.select_members_for_cycle(&members, MarketScope::KoreaShortTerm, None, &policy);

    assert_eq!(market_committee_layouts().len(), 6);
    assert!(plan.selected_member_ids.len() <= policy.max_active_members_per_cycle);
    assert!(plan.selected_member_ids.len() <= policy.max_active_members_per_market_scope);
    assert_ne!(plan.selected_member_ids.len(), 18);
    assert!(
        plan.selected_member_ids
            .contains(&"risk-member".to_string())
    );
    assert!(
        plan.skipped_members
            .iter()
            .any(|skip| { skip.reason == MemberSelectionSkipReason::OverActivationLimit })
    );
    assert!(
        plan.policy_notes
            .iter()
            .any(|note| note.contains("do not run 18 AI cores concurrently"))
    );

    let mut mixed_scope_members = members[..4].to_vec();
    mixed_scope_members[0].market_scopes = vec![MarketScope::KoreaLongTerm];
    let mixed_registry = AiMemberCoreRegistry::from_members(&mixed_scope_members);
    let scoped_policy = MemberActivationPolicy {
        max_active_members_per_cycle: 5,
        max_active_members_per_market_scope: 1,
        ..MemberActivationPolicy::default()
    };
    assert_eq!(
        mixed_registry.active_core_count(&scoped_policy, MarketScope::KoreaShortTerm),
        1
    );
    assert_eq!(
        mixed_registry.active_core_count(&scoped_policy, MarketScope::KoreaLongTerm),
        1
    );
    assert_eq!(
        mixed_registry.estimate_cycle_memory_mb(&scoped_policy, MarketScope::UsLongTerm),
        0
    );
}

#[test]
fn core_aware_brain_respects_deferred_offline_and_mock_modes() {
    let member = AICommitteeMember {
        member_id: "core-member".to_string(),
        display_name: "core member".to_string(),
        market_scopes: vec![MarketScope::KoreaShortTerm],
        style_profile: "trend".to_string(),
        voice_weight: 0.7,
        score: 0.6,
        status: AICommitteeMemberStatus::Active,
        runtime_mode: AIRuntimeMode::MockLocal,
        core_spec: None,
        role: Some(IndependentMemberRole::TrendEntry),
        memory_state: None,
    };
    let packet = MemberInputPacket {
        member_id: member.member_id.clone(),
        market_data: serde_json::from_value(serde_json::json!({
            "symbol": "005930.KS",
            "market_scope": "KoreaShortTerm",
            "timestamp": "2026-05-21T09:00:00+09:00",
            "price": 78000.0,
            "change_pct": 4.2,
            "volume": 1000.0,
            "volatility_hint": 0.03,
            "source_label": "test"
        }))
        .expect("market data"),
        news: vec![
            serde_json::from_value(serde_json::json!({
                "symbol": "005930.KS",
                "headline": "positive",
                "summary": "positive",
                "sentiment_hint": "positive",
                "source_label": "test",
                "timestamp": "2026-05-21T09:00:00+09:00"
            }))
            .expect("news"),
        ],
        owner_context: Some("paper-only".to_string()),
        previous_member_score: Some(0.6),
    };
    let offline_adapter = OfflineMemberBrainAdapter {
        fixtures: vec![OfflineMemberOpinionFixture {
            member_id: member.member_id.clone(),
            symbol: packet.market_data.symbol.clone(),
            market_scope: packet.market_data.market_scope,
            stance: MemberStance::BuyProposal,
            confidence: 0.8,
            expected_return_hint: 0.03,
            risk_hint: 0.04,
            evidence_notes: vec!["offline fixture opinion".to_string()],
            event_triggered: true,
            event_reason: Some("offline fixture".to_string()),
        }],
    };

    let offline = CoreAwareMemberBrainAdapter {
        member: member.clone(),
        core_spec: Mamba3GatedDeltaNetCoreSpec::offline_fixture_for(&member.member_id),
        offline_adapter: offline_adapter.clone(),
    }
    .produce_opinion(&packet);
    assert_eq!(offline.stance, MemberStance::BuyProposal);

    let mock = CoreAwareMemberBrainAdapter {
        member: member.clone(),
        core_spec: Mamba3GatedDeltaNetCoreSpec::mock_local_for(&member.member_id),
        offline_adapter: offline_adapter.clone(),
    }
    .produce_opinion(&packet);
    assert_eq!(mock.stance, MemberStance::BuyProposal);
    assert!(
        mock.evidence_notes
            .iter()
            .any(|note| note.contains("deterministic mock"))
    );

    let runtime_deferred = CoreAwareMemberBrainAdapter {
        member: member.clone(),
        core_spec: Mamba3GatedDeltaNetCoreSpec::runtime_deferred_for(&member.member_id),
        offline_adapter: offline_adapter.clone(),
    }
    .produce_opinion(&packet);
    assert_eq!(runtime_deferred.stance, MemberStance::NeedMoreEvidence);
    assert_eq!(
        runtime_deferred.event_reason.as_deref(),
        Some("runtime deferred")
    );

    let training_deferred = CoreAwareMemberBrainAdapter {
        member,
        core_spec: Mamba3GatedDeltaNetCoreSpec::training_deferred_for("core-member"),
        offline_adapter,
    }
    .produce_opinion(&packet);
    assert_eq!(training_deferred.stance, MemberStance::NeedMoreEvidence);
    assert_eq!(
        training_deferred.event_reason.as_deref(),
        Some("training deferred")
    );
}

#[test]
fn data_router_routes_all_market_scopes_without_creating_opinions() {
    let text = std::fs::read_to_string("examples/minimal_ai_committee_multi_market_sample.json")
        .expect("multi-market sample");
    let input: DataRouterInput = serde_json::from_str(&text).expect("data router input");
    let market_count = input.market_data.len();
    let member_count = input.members.len();
    let output = route_data_to_ai_members(input.clone());

    assert_eq!(market_count, 6);
    assert_eq!(member_count, 7);
    assert_eq!(output.unrouted_symbol_count, 0);
    assert_eq!(output.routed_member_count, 6);
    assert_eq!(output.packets.len(), 6);
    assert!(
        output
            .safety_notes
            .iter()
            .any(|note| note.contains("does not create opinions"))
    );
    assert!(
        output
            .safety_notes
            .iter()
            .any(|note| note.contains("AI members judge"))
    );
    assert!(output.safety_notes.iter().any(|note| {
        let note = note.to_ascii_lowercase();
        note.contains("does not create") && note.contains("recommendations")
    }));

    let routed_scopes: std::collections::BTreeSet<_> = output
        .packets
        .iter()
        .map(|packet| format!("{:?}", packet.market_data.market_scope))
        .collect();
    assert_eq!(
        routed_scopes,
        [
            "KoreaShortTerm",
            "KoreaLongTerm",
            "UsShortTerm",
            "UsLongTerm",
            "CryptoShortTerm",
            "CryptoLongTerm",
        ]
        .into_iter()
        .map(str::to_string)
        .collect()
    );
    assert!(
        !output
            .packets
            .iter()
            .any(|packet| packet.member_id == "disabled-all-market")
    );
    for packet in &output.packets {
        let member = input
            .members
            .iter()
            .find(|member| member.member_id == packet.member_id)
            .expect("packet member exists");
        assert!(
            member
                .market_scopes
                .contains(&packet.market_data.market_scope)
        );
        assert!(
            packet
                .news
                .iter()
                .all(|news| news.symbol == packet.market_data.symbol)
        );
    }

    let first_packet = output
        .packets
        .iter()
        .find(|packet| packet.member_id == "kr-short-trend")
        .expect("kr short packet");
    let offline_adapter = OfflineMemberBrainAdapter {
        fixtures: vec![OfflineMemberOpinionFixture {
            member_id: first_packet.member_id.clone(),
            symbol: first_packet.market_data.symbol.clone(),
            market_scope: first_packet.market_data.market_scope,
            stance: MemberStance::BuyProposal,
            confidence: 0.71,
            expected_return_hint: 0.02,
            risk_hint: first_packet.market_data.volatility_hint,
            evidence_notes: vec!["offline fixture opinion".to_string()],
            event_triggered: true,
            event_reason: Some("router-fed offline fixture".to_string()),
        }],
    };
    let offline_opinion = offline_adapter.produce_opinion(first_packet);
    assert_eq!(offline_opinion.member_id, "kr-short-trend");
    assert_eq!(offline_opinion.stance, MemberStance::BuyProposal);
    assert!(offline_opinion.event_triggered);
    assert!(
        offline_opinion
            .evidence_notes
            .contains(&"offline fixture opinion".to_string())
    );

    let member = input
        .members
        .iter()
        .find(|member| member.member_id == first_packet.member_id)
        .expect("matching member")
        .clone();
    let opinion = DeterministicMockBrain { member }.produce_opinion(first_packet);
    assert_eq!(opinion.member_id, "kr-short-trend");
    assert_eq!(opinion.stance, MemberStance::BuyProposal);
    assert!(opinion.event_triggered);
}

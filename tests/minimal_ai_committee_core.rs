use std::process::Command;

use soma_zero::league::minimal_ai_committee_core::{
    AICommitteeMember, AICommitteeMemberStatus, AIRuntimeMode, AiMemberBrain, AiMemberCoreRegistry,
    ArchetypeRiskBias, ArchetypeStyleCardRegistry, ArchetypeStyleTag, AutonomousPaperCycleMode,
    BatchCommitteeCycleInput, BatchCommitteeCycleWithStateInput, ChairmanFinalAction,
    CommitteeStateExportConfig, CommitteeStateExportInput, CommitteeStateSnapshotSource,
    CoreAwareMemberBrainAdapter, CoreRuntimeStatus, DailyBriefStorageUpdateInput, DataRouterInput,
    DeterministicMockBrain, EvidencePreference, IndependentMemberRole, InvestmentEventQueue,
    InvestorArchetypeStyleCard, Mamba3GatedDeltaNetCoreSpec, MarketScope, MemberActivationPolicy,
    MemberCoreFamily, MemberInputPacket, MemberLearningSignal, MemberScoreUpdateReason,
    MemberSelectionSkipReason, MemberStance, MemberStateStore, MemberStyleStatus, MemoryCoreKind,
    MinimalAiCommitteeCycleConfig, NextActionType, NextOwnerActionType, OfflineMemberBrainAdapter,
    OfflineMemberOpinionFixture, OfflineMemberOutputBatch, OwnerActionAfterApplyFilePolicy,
    OwnerActionApplyMode, OwnerActionComposerConfig, OwnerActionConsumptionConfig,
    OwnerActionDuplicatePolicy, OwnerActionFile, OwnerActionProcessedStatus, OwnerAttentionAction,
    OwnerAttentionActionSafetyStatus, OwnerAttentionActionType, OwnerAttentionInbox,
    OwnerAttentionInboxStatus, OwnerAttentionPriority, OwnerAttentionQueue,
    OwnerAttentionTriageInput, OwnerAttentionType, OwnerConfirmationPolicy,
    OwnerConsoleTerminalOptions, OwnerDailyBriefStore, OwnerFeedbackOutcome, OwnerFeedbackType,
    PreferredMarketBias, PreferredTimeHorizon, RealArchetypeIntakePolicy, RiskGovernorStatus,
    SequenceCoreKind, SnapshotFileNamingPolicy, SourceConfidence, StyleCardStatus,
    StyleMappingMode, WatchlistCandidate, WatchlistCandidateStatus, WatchlistCandidateStore,
    WatchlistRecheckConfig, WatchlistRecheckSkipReason, build_committee_state_snapshot,
    build_owner_console_read_model, compose_owner_action_from_read_model,
    consume_owner_action_file, consume_owner_action_file_with_previous_run,
    create_three_member_pilot_roster, load_owner_attention_actions_from_local_json,
    load_owner_console_read_model_from_local_file, load_owner_feedback_from_local_json,
    mac_mini_local_policy, map_style_cards_to_three_member_pilot, market_committee_layouts,
    render_owner_console_terminal_view, route_data_to_ai_members,
    run_autonomous_paper_committee_loop_from_config_path,
    run_batch_committee_cycle_from_config_path, run_batch_committee_cycle_with_state,
    run_batch_committee_cycle_with_state_from_config_path, run_daily_brief_storage_update,
    run_minimal_committee_cycle, run_owner_attention_triage, run_owner_console_viewer,
    run_watchlist_recheck_cycle, save_committee_state_snapshot, write_committee_state_export,
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
        emit_owner_console_view: false,
        owner_feedback_path: None,
        emit_reconsideration_view: false,
        autonomous_paper_run: false,
        run_id: None,
        market_scopes: Vec::new(),
        symbols: Vec::new(),
        max_cycles: 1,
        cycle_mode: AutonomousPaperCycleMode::SingleShot,
        require_owner_confirmation: OwnerConfirmationPolicy::Never,
        local_market_data_path: None,
        local_news_path: None,
        paper_only: true,
        owner_attention_inbox_input_path: None,
        owner_attention_inbox_output_path: None,
        owner_attention_actions_path: None,
        watchlist_candidate_input_path: None,
        watchlist_candidate_output_path: None,
        emit_owner_attention_inbox: false,
        enable_watchlist_recheck: false,
        watchlist_input_path: None,
        watchlist_output_path: None,
        max_candidates_per_cycle: 3,
        include_risk_blocked: false,
        include_needs_evidence: true,
        emit_owner_daily_brief: false,
        owner_daily_brief_store_input_path: None,
        owner_daily_brief_store_output_path: None,
        committee_state_snapshot_output_path: None,
        emit_committee_state_snapshot: false,
        committee_state_export_root_path: None,
        write_latest_snapshot: false,
        write_history_snapshot: false,
        write_snapshot_index: false,
        write_owner_console_read_model: false,
        committee_state_schema_version: None,
        max_snapshot_history_entries: None,
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
    assert!(stdout.contains("\"owner_console_view\""));
    assert!(stdout.contains("\"owner_feedback_reconsideration\""));
    assert!(stdout.contains("\"attention_queue\""));
    assert!(stdout.contains("\"owner_attention_triage\""));
    assert!(stdout.contains("\"generated_owner_feedback_count\""));
    assert!(stdout.contains("\"generated_watchlist_candidate_count\""));
    assert!(stdout.contains("\"generated_watchlist_candidates\""));
    assert!(stdout.contains("\"watchlist_recheck\""));
    assert!(stdout.contains("\"owner_daily_brief\""));
    assert!(stdout.contains("\"lifecycle_events\""));
    assert!(stdout.contains("\"paper_decision_archive\""));
    assert!(stdout.contains("\"member_status_rows\""));
    assert!(stdout.contains("\"next_action_rows\""));
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
        emit_owner_console_view: false,
        owner_feedback_path: None,
        emit_reconsideration_view: false,
        autonomous_paper_run: false,
        run_id: None,
        market_scopes: Vec::new(),
        symbols: Vec::new(),
        max_cycles: 1,
        cycle_mode: AutonomousPaperCycleMode::SingleShot,
        require_owner_confirmation: OwnerConfirmationPolicy::Never,
        local_market_data_path: None,
        local_news_path: None,
        paper_only: true,
        owner_attention_inbox_input_path: None,
        owner_attention_inbox_output_path: None,
        owner_attention_actions_path: None,
        watchlist_candidate_input_path: None,
        watchlist_candidate_output_path: None,
        emit_owner_attention_inbox: false,
        enable_watchlist_recheck: false,
        watchlist_input_path: None,
        watchlist_output_path: None,
        max_candidates_per_cycle: 3,
        include_risk_blocked: false,
        include_needs_evidence: true,
        emit_owner_daily_brief: false,
        owner_daily_brief_store_input_path: None,
        owner_daily_brief_store_output_path: None,
        committee_state_snapshot_output_path: None,
        emit_committee_state_snapshot: false,
        committee_state_export_root_path: None,
        write_latest_snapshot: false,
        write_history_snapshot: false,
        write_snapshot_index: false,
        write_owner_console_read_model: false,
        committee_state_schema_version: None,
        max_snapshot_history_entries: None,
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
    let initial_store = store.clone();
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
        emit_owner_console_view: true,
        owner_feedback: Vec::new(),
        emit_reconsideration_view: false,
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
        let initial_state = initial_store
            .get_member_state(&state.member_id)
            .expect("initial member state");
        let (expected_score, expected_voice_weight) = result
            .batch_result
            .score_updates
            .iter()
            .filter(|update| update.member_id == state.member_id)
            .fold(
                (initial_state.score, initial_state.voice_weight),
                |(score, voice_weight), update| {
                    (
                        (score + update.new_score - update.previous_score).clamp(0.0, 1.0),
                        (voice_weight + update.new_voice_weight - update.previous_voice_weight)
                            .clamp(0.0, 1.0),
                    )
                },
            );
        assert!((state.score - expected_score).abs() < 1e-12);
        assert!((state.voice_weight - expected_voice_weight).abs() < 1e-12);
    }
    assert_eq!(result.state_update.updated_member_states, store.members);
    assert!(risk_state.memory_state.recent_opinion_count > 0);
    assert!(risk_state.memory_state.recent_event_count > 0);
    assert!(
        risk_state.score
            > initial_store
                .get_member_state("risk-kr-short")
                .expect("initial risk state")
                .score
    );
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
    let console = first
        .owner_console_view
        .as_ref()
        .expect("owner console view");
    assert_eq!(console.cycle_id, first.batch_result.batch_id);
    assert_eq!(console.member_status_rows.len(), 3);
    assert_eq!(console.active_members.len(), 3);
    assert!(!console.event_rows.is_empty());
    assert_eq!(
        console.event_rows.len(),
        first.batch_result.event_queue.event_count
    );
    assert_eq!(
        console.committee_rows.len(),
        first.batch_result.committee_sessions.len()
    );
    assert_eq!(
        console.chairman_decision_rows.len(),
        first.batch_result.chairman_decisions.len()
    );
    assert!(!console.risk_veto_rows.is_empty());
    assert!(!console.voice_change_rows.is_empty());
    assert!(
        console
            .next_action_rows
            .iter()
            .any(|row| row.action_type == NextActionType::RiskBlocked)
    );
    assert!(
        console
            .next_action_rows
            .iter()
            .any(|row| row.action_type == NextActionType::NeedMoreEvidence)
    );
    assert!(
        console
            .next_action_rows
            .iter()
            .any(|row| row.action_type == NextActionType::Watch)
    );
    assert!(console.paper_only_warning.contains("paper-only"));
    let console_json = serde_json::to_string(console).expect("console json");
    for forbidden_field in ["\"broker\"", "\"order\"", "\"account\""] {
        assert!(!console_json.contains(forbidden_field));
    }
    assert!(first.batch_result.safety_summary.no_broker_order_account);
    assert!(first.batch_result.safety_summary.no_model_training);
    assert!(first.batch_result.safety_summary.no_live_inference);
}

#[test]
fn owner_feedback_routes_reconsideration_and_preserves_paper_only_safety() {
    let loaded_feedback = load_owner_feedback_from_local_json(std::path::Path::new(
        "examples/minimal_owner_feedback.sample.json",
    ))
    .expect("load owner feedback sample");
    assert_eq!(loaded_feedback.len(), 7);
    assert!(loaded_feedback.iter().all(|feedback| feedback.paper_only));

    let err = load_owner_feedback_from_local_json(std::path::Path::new(
        "https://example.invalid/owner-feedback.json",
    ))
    .expect_err("remote owner feedback must fail");
    assert!(err.contains("must be local"));

    for (case, text) in [
        ("trade", "execute trade with real money"),
        ("broker", "send this to broker account"),
        ("claim", "use private data for guaranteed return"),
    ] {
        let unsafe_path = std::path::PathBuf::from(format!(
            "target/sprint134_unsafe_owner_feedback_{case}.json"
        ));
        let unsafe_feedback = serde_json::json!([
            {
                "feedback_id": format!("unsafe-owner-feedback-{case}"),
                "symbol": "BTCUSDT",
                "market_scope": "CryptoShortTerm",
                "target_member_id": null,
                "feedback_type": "RiskConcern",
                "text": text,
                "priority": "High",
                "created_at": "2026-05-22T18:04:00Z",
                "paper_only": true
            }
        ])
        .to_string();
        std::fs::write(&unsafe_path, unsafe_feedback).expect("write unsafe owner feedback");
        let err = load_owner_feedback_from_local_json(&unsafe_path)
            .expect_err("unsafe owner feedback text must fail");
        assert!(err.contains("unsafe"));
        let _ = std::fs::remove_file(&unsafe_path);
    }

    let first = run_batch_committee_cycle_with_state_from_config_path(std::path::Path::new(
        "examples/soma_minimal_ai_committee_core.toml",
    ))
    .expect("first feedback reconsideration run");
    let second = run_batch_committee_cycle_with_state_from_config_path(std::path::Path::new(
        "examples/soma_minimal_ai_committee_core.toml",
    ))
    .expect("second feedback reconsideration run");
    assert_eq!(first, second);

    let reconsideration = first
        .owner_feedback_reconsideration
        .as_ref()
        .expect("owner feedback reconsideration");
    assert_eq!(reconsideration.owner_feedback_count, loaded_feedback.len());
    assert!(
        reconsideration
            .paper_only_warning
            .contains("no broker/order/account")
    );
    assert!(
        reconsideration
            .owner_feedback_journal_entries
            .iter()
            .any(|entry| {
                entry.feedback_id == "owner-feedback-comment-samsung"
                    && !entry.reconsideration_opened
                    && entry.outcome == OwnerFeedbackOutcome::LoggedOnly
            })
    );
    assert!(
        reconsideration
            .owner_feedback_journal_entries
            .iter()
            .any(|entry| {
                entry.feedback_id == "owner-feedback-outcome-samsung"
                    && !entry.reconsideration_opened
                    && entry.outcome == OwnerFeedbackOutcome::LoggedOnly
                    && entry.note.contains("paper outcome label")
            })
    );
    assert!(
        reconsideration
            .owner_feedback_journal_entries
            .iter()
            .any(|entry| {
                entry.feedback_id == "owner-feedback-disagree-samsung-trend"
                    && entry.routed_to_members == vec!["trend-kr-short".to_string()]
                    && entry.reconsideration_opened
            })
    );
    assert!(
        reconsideration
            .reconsideration_sessions
            .iter()
            .any(|session| {
                session.owner_feedback.feedback_type == OwnerFeedbackType::RiskConcern
                    && session
                        .risk_flags
                        .contains(&"owner_risk_concern".to_string())
            })
    );
    assert!(
        reconsideration
            .reconsideration_sessions
            .iter()
            .any(|session| {
                session.owner_feedback.feedback_type == OwnerFeedbackType::ReconsiderationRequest
                    && !session.invited_members.is_empty()
            })
    );
    assert!(
        reconsideration
            .routed_feedback_packets
            .iter()
            .any(|packet| {
                packet.feedback.feedback_type == OwnerFeedbackType::EvidenceRequest
                    && packet
                        .related_previous_opinions
                        .iter()
                        .any(|opinion| opinion.member_id.contains("evidence"))
            })
    );
    assert!(
        reconsideration
            .revised_member_opinions
            .iter()
            .any(|opinion| {
                opinion.member_id == "trend-kr-short"
                    && opinion.previous_stance == MemberStance::BuyProposal
                    && opinion.revised_stance == MemberStance::Hold
                    && opinion.changed
            })
    );
    assert!(
        reconsideration
            .revised_member_opinions
            .iter()
            .any(|opinion| {
                opinion.member_id.contains("evidence")
                    && opinion.revised_stance == MemberStance::NeedMoreEvidence
                    && !opinion.evidence_needed.is_empty()
            })
    );
    assert!(
        reconsideration
            .revised_member_opinions
            .iter()
            .any(|opinion| {
                opinion.member_id.contains("risk")
                    && opinion.revised_stance == MemberStance::NoTrade
                    && !opinion.risk_notes.is_empty()
            })
    );
    assert!(
        reconsideration
            .chairman_reconsideration_decisions
            .iter()
            .any(|decision| decision.risk_governor_status == RiskGovernorStatus::Vetoed)
    );
    assert!(
        reconsideration
            .chairman_reconsideration_decisions
            .iter()
            .any(|decision| {
                decision.final_action
                    == soma_zero::league::minimal_ai_committee_core::ChairmanReconsiderationFinalAction::NeedMoreEvidence
            })
    );
    assert!(
        reconsideration
            .chairman_reconsideration_decisions
            .iter()
            .any(|decision| {
                decision.final_action
                    == soma_zero::league::minimal_ai_committee_core::ChairmanReconsiderationFinalAction::PaperHold
            })
    );
    assert!(
        reconsideration
            .updated_owner_console_view
            .next_action_rows
            .iter()
            .any(|row| row.action_type == NextActionType::RiskBlocked)
    );

    let json = serde_json::to_string(reconsideration).expect("reconsideration json");
    for forbidden_field in ["\"broker\"", "\"account\""] {
        assert!(!json.contains(forbidden_field));
    }
    assert!(first.batch_result.safety_summary.no_broker_order_account);
    assert!(first.batch_result.safety_summary.no_model_training);
    assert!(first.batch_result.safety_summary.no_live_inference);
}

#[test]
fn autonomous_paper_loop_runs_cycles_attention_queue_and_archive_safely() {
    let sample_watchlist = WatchlistCandidateStore::load_from_local_json(std::path::Path::new(
        "examples/minimal_watchlist_candidates.sample.json",
    ))
    .expect("load watchlist sample");
    assert!(sample_watchlist.active_count >= 1);
    assert!(
        sample_watchlist
            .candidates
            .iter()
            .all(|candidate| candidate.paper_only)
    );

    let first = run_autonomous_paper_committee_loop_from_config_path(std::path::Path::new(
        "examples/soma_minimal_ai_committee_core.toml",
    ))
    .expect("first autonomous run");
    let second = run_autonomous_paper_committee_loop_from_config_path(std::path::Path::new(
        "examples/soma_minimal_ai_committee_core.toml",
    ))
    .expect("second autonomous run");
    assert_eq!(first, second);

    assert_eq!(first.run_id, "soma-autonomous-paper-sprint135");
    assert_eq!(first.cycle_count, 1);
    assert_eq!(first.cycles.len(), 1);
    assert!(!first.final_member_states.is_empty());
    assert!(!first.cycles[0].owner_summary.symbols_reviewed.is_empty());
    assert!(first.cycles[0].owner_console_view.is_some());
    assert!(
        first
            .attention_queue
            .items
            .iter()
            .any(|item| item.attention_type == OwnerAttentionType::RiskVeto)
    );
    assert!(
        first
            .attention_queue
            .items
            .iter()
            .any(|item| item.attention_type == OwnerAttentionType::NeedMoreEvidence)
    );
    let mut paper_hold_result = first.cycles[0].batch_result.clone();
    let paper_hold_decision = paper_hold_result
        .chairman_decisions
        .first_mut()
        .expect("paper hold fixture decision");
    paper_hold_decision.final_action = ChairmanFinalAction::PaperHold;
    paper_hold_decision.risk_governor_status = RiskGovernorStatus::Passed;
    let paper_hold_queue = OwnerAttentionQueue::from_batch_cycle_result(
        "paper-hold-watchlist-test",
        0,
        &paper_hold_result,
        None,
        OwnerConfirmationPolicy::Never,
    );
    assert!(
        paper_hold_queue
            .items
            .iter()
            .any(|item| item.attention_type == OwnerAttentionType::WatchlistCandidate)
    );
    assert_eq!(
        first
            .attention_queue
            .items
            .first()
            .map(|item| item.priority),
        Some(OwnerAttentionPriority::High)
    );
    assert_eq!(first.attention_queue.requires_owner_input_count, 0);
    assert!(first.attention_queue.unresolved_items().is_empty());
    assert!(!first.paper_decision_archive.entries.is_empty());
    assert!(first.paper_decision_archive.risk_veto_count() >= 1);
    assert!(
        first
            .paper_decision_archive
            .entries
            .iter()
            .all(|entry| entry.paper_only)
    );
    assert!(
        !first
            .paper_decision_archive
            .decisions_by_symbol("BTCUSDT")
            .is_empty()
    );
    assert!(first.safety_summary.no_broker_order_account);
    assert!(first.safety_summary.no_model_training);
    assert!(first.safety_summary.no_live_inference);
    let triage = first
        .owner_attention_triage
        .as_ref()
        .expect("owner attention triage");
    assert!(triage.inbox.open_count <= triage.inbox.items.len());
    assert!(!triage.action_results.is_empty());
    assert!(!triage.generated_owner_feedback.is_empty());
    assert!(!triage.generated_watchlist_candidates.is_empty());
    let recheck = first.watchlist_recheck.as_ref().expect("watchlist recheck");
    assert_eq!(
        recheck.recheck_id,
        "soma-autonomous-paper-sprint135-watchlist-recheck"
    );
    assert_eq!(recheck.selection.selected_count, 3);
    assert!(
        recheck
            .selection
            .skip_reasons
            .contains(&WatchlistRecheckSkipReason::Archived)
    );
    assert!(
        recheck
            .selection
            .skip_reasons
            .contains(&WatchlistRecheckSkipReason::OverCandidateLimit)
    );
    assert!(
        recheck
            .selection
            .skip_reasons
            .contains(&WatchlistRecheckSkipReason::RiskBlockedExcluded)
    );
    assert!(recheck.batch_result.member_opinions.iter().all(|opinion| {
        recheck.selected_candidates.iter().any(|candidate| {
            candidate.symbol == opinion.symbol && candidate.market_scope == opinion.market_scope
        })
    }));
    assert!(
        recheck
            .lifecycle_events
            .iter()
            .any(|event| event.new_status == WatchlistCandidateStatus::RiskBlocked)
    );
    assert!(
        recheck
            .lifecycle_events
            .iter()
            .any(|event| event.new_status == WatchlistCandidateStatus::NeedsEvidence)
    );
    assert!(
        recheck
            .lifecycle_events
            .iter()
            .all(|event| event.paper_only)
    );
    assert!(!recheck.generated_attention_items.is_empty());
    let brief = recheck
        .owner_daily_brief
        .as_ref()
        .expect("owner daily brief");
    assert!(!brief.reviewed_symbols.is_empty());
    assert!(!brief.risk_vetoes.is_empty());
    assert!(!brief.need_more_evidence_items.is_empty());
    assert!(!brief.next_owner_attention.is_empty());
    assert!(brief.brief_text.contains("paper-only"));
    assert!(recheck.safety_summary.no_broker_order_account);
    assert!(recheck.safety_summary.no_model_training);
    assert!(recheck.safety_summary.no_live_inference);

    let fixed_config_path = std::path::Path::new("target/sprint135_autonomous_fixed.toml");
    std::fs::write(
        fixed_config_path,
        r#"
input_path = "examples/minimal_ai_committee_multi_market_sample.json"
offline_member_output_batch_path = "examples/minimal_offline_member_output_batch.sample.json"
batch_mode = true
autonomous_paper_run = true
run_id = "sprint135-fixed-test"
max_cycles = 2
cycle_mode = "FixedCount"
require_owner_confirmation = "Never"
emit_owner_console_view = true
pilot_roster = "three_member"
paper_outcome = "Positive"
archetype_style_cards_path = "examples/investor_archetype_style_cards.sample.json"
style_mapping_mode = "LocalFixture"
"#,
    )
    .expect("write fixed autonomous config");
    let fixed = run_autonomous_paper_committee_loop_from_config_path(fixed_config_path)
        .expect("fixed autonomous run");
    assert_eq!(fixed.cycle_count, 2);
    assert!(
        fixed
            .attention_queue
            .items
            .iter()
            .all(|item| item.attention_type != OwnerAttentionType::OwnerFeedbackAvailable)
    );
    let first_risk_state = fixed.cycles[0]
        .state_update
        .updated_member_states
        .iter()
        .find(|state| state.member_id == "risk-kr-short")
        .expect("first risk state");
    let second_risk_state = fixed.cycles[1]
        .state_update
        .updated_member_states
        .iter()
        .find(|state| state.member_id == "risk-kr-short")
        .expect("second risk state");
    assert!(
        second_risk_state.memory_state.recent_opinion_count
            >= first_risk_state.memory_state.recent_opinion_count
    );
    assert_eq!(fixed.attention_queue.requires_owner_input_count, 0);
    let _ = std::fs::remove_file(fixed_config_path);
}

#[test]
fn watchlist_recheck_direct_cycle_loads_local_paths() {
    let config = MinimalAiCommitteeCycleConfig::from_toml_path(std::path::Path::new(
        "examples/soma_minimal_ai_committee_core.toml",
    ))
    .expect("config");
    let loaded_batch = config.load_batch_input().expect("batch input");
    std::fs::create_dir_all("target").expect("target dir");
    let market_data_path = std::path::Path::new("target/sprint137_watchlist_market_data.json");
    let news_path = std::path::Path::new("target/sprint137_watchlist_news.json");
    std::fs::write(
        market_data_path,
        serde_json::to_string_pretty(&loaded_batch.market_data).expect("market data json"),
    )
    .expect("write market data");
    std::fs::write(
        news_path,
        serde_json::to_string_pretty(&loaded_batch.news).expect("news json"),
    )
    .expect("write news");

    let mut seed_batch = loaded_batch;
    seed_batch.market_data.clear();
    seed_batch.news.clear();
    seed_batch.offline_output_batch = None;

    let result = run_watchlist_recheck_cycle(WatchlistRecheckConfig {
        recheck_id: "direct-path-watchlist-recheck".to_string(),
        watchlist_input_path: Some("examples/minimal_watchlist_candidates.sample.json".to_string()),
        watchlist_output_path: None,
        member_state_input_path: None,
        member_state_output_path: None,
        market_data_path: Some(market_data_path.to_string_lossy().to_string()),
        news_path: Some(news_path.to_string_lossy().to_string()),
        offline_member_output_batch_path: Some(
            "examples/minimal_offline_member_output_batch.sample.json".to_string(),
        ),
        max_candidates_per_cycle: 3,
        include_risk_blocked: false,
        include_needs_evidence: true,
        emit_owner_daily_brief: true,
        paper_only: true,
        watchlist_store: WatchlistCandidateStore::new("ignored-direct-path-store"),
        batch_input: seed_batch,
        member_state_store: None,
    })
    .expect("direct path watchlist recheck");

    assert_eq!(result.selection.selected_count, 3);
    assert!(
        result
            .selection
            .skip_reasons
            .contains(&WatchlistRecheckSkipReason::Archived)
    );
    assert!(result.batch_result.member_opinions.iter().any(|opinion| {
        opinion
            .evidence_notes
            .iter()
            .any(|note| note == "offline batch opinion")
    }));
    assert!(result.lifecycle_events.iter().any(|event| {
        event.new_status == WatchlistCandidateStatus::RiskBlocked
            || event.new_status == WatchlistCandidateStatus::NeedsEvidence
    }));
    let brief = result.owner_daily_brief.expect("owner daily brief");
    assert!(brief.reviewed_symbols.contains(&"BTCUSDT".to_string()));
    assert!(brief.brief_text.contains("paper-only"));

    let _ = std::fs::remove_file(market_data_path);
    let _ = std::fs::remove_file(news_path);
}

#[test]
fn watchlist_recheck_updates_paper_candidate_without_order_path() {
    let store = WatchlistCandidateStore {
        store_id: "paper-candidate-recheck-store".to_string(),
        candidates: vec![WatchlistCandidate {
            candidate_id: "watchlist-paper-samsung".to_string(),
            symbol: "005930.KS".to_string(),
            market_scope: MarketScope::KoreaShortTerm,
            source_attention_item_id: "attention-paper-candidate".to_string(),
            reason: "Paper-only candidate path test".to_string(),
            status: WatchlistCandidateStatus::Watching,
            created_at: Some("2026-05-23T09:40:00Z".to_string()),
            paper_only: true,
        }],
        active_count: 1,
        risk_blocked_count: 0,
        needs_evidence_count: 0,
        paper_only: true,
    };
    let mut batch_input = MinimalAiCommitteeCycleConfig::from_toml_path(std::path::Path::new(
        "examples/soma_minimal_ai_committee_core.toml",
    ))
    .expect("config")
    .load_batch_input()
    .expect("batch input");
    batch_input
        .market_data
        .retain(|market| market.symbol == "005930.KS");
    batch_input.news.retain(|news| news.symbol == "005930.KS");
    batch_input.offline_output_batch = Some(OfflineMemberOutputBatch {
        batch_id: "paper-candidate-offline-batch".to_string(),
        created_at: "2026-05-23T09:41:00Z".to_string(),
        source_label: "unit-test".to_string(),
        opinions: vec![
            OfflineMemberOpinionFixture {
                member_id: "trend-kr-short".to_string(),
                symbol: "005930.KS".to_string(),
                market_scope: MarketScope::KoreaShortTerm,
                stance: MemberStance::BuyProposal,
                confidence: 0.91,
                expected_return_hint: 0.03,
                risk_hint: 0.02,
                evidence_notes: vec!["paper-only buy candidate".to_string()],
                event_triggered: true,
                event_reason: Some("paper-only candidate".to_string()),
            },
            OfflineMemberOpinionFixture {
                member_id: "risk-kr-short".to_string(),
                symbol: "005930.KS".to_string(),
                market_scope: MarketScope::KoreaShortTerm,
                stance: MemberStance::Hold,
                confidence: 0.2,
                expected_return_hint: 0.0,
                risk_hint: 0.01,
                evidence_notes: vec!["paper-only risk passed".to_string()],
                event_triggered: false,
                event_reason: None,
            },
            OfflineMemberOpinionFixture {
                member_id: "evidence-kr-short".to_string(),
                symbol: "005930.KS".to_string(),
                market_scope: MarketScope::KoreaShortTerm,
                stance: MemberStance::Hold,
                confidence: 0.2,
                expected_return_hint: 0.0,
                risk_hint: 0.01,
                evidence_notes: vec!["paper-only evidence enough".to_string()],
                event_triggered: false,
                event_reason: None,
            },
        ],
    });
    let result = run_watchlist_recheck_cycle(WatchlistRecheckConfig {
        recheck_id: "paper-candidate-recheck".to_string(),
        watchlist_input_path: None,
        watchlist_output_path: None,
        member_state_input_path: None,
        member_state_output_path: None,
        market_data_path: None,
        news_path: None,
        offline_member_output_batch_path: None,
        max_candidates_per_cycle: 1,
        include_risk_blocked: false,
        include_needs_evidence: true,
        emit_owner_daily_brief: true,
        paper_only: true,
        watchlist_store: store,
        batch_input,
        member_state_store: None,
    })
    .expect("paper candidate recheck");
    assert!(
        result
            .lifecycle_events
            .iter()
            .any(|event| event.new_status == WatchlistCandidateStatus::PaperCandidate)
    );
    assert!(
        result
            .updated_watchlist_store
            .candidates
            .iter()
            .all(|candidate| candidate.paper_only)
    );
    let json = serde_json::to_string(&result).expect("watchlist recheck json");
    for forbidden_field in ["\"broker\"", "\"order\"", "\"account\""] {
        assert!(!json.contains(forbidden_field));
    }
}

#[test]
fn owner_attention_inbox_triages_actions_and_watchlist_safely() {
    let run = run_autonomous_paper_committee_loop_from_config_path(std::path::Path::new(
        "examples/soma_minimal_ai_committee_core.toml",
    ))
    .expect("autonomous run with attention inbox");
    let mut inbox = OwnerAttentionInbox::from_attention_queue(&run.attention_queue);
    let original_count = inbox.items.len();
    inbox.merge_new_items(&run.attention_queue);
    assert_eq!(inbox.items.len(), original_count);
    assert_eq!(
        inbox
            .high_priority_items()
            .first()
            .map(|item| item.priority),
        Some(OwnerAttentionPriority::High)
    );
    assert!(!inbox.open_items().is_empty());
    assert_eq!(inbox.requires_owner_input_count, 0);
    assert!(inbox.items_requiring_owner_input().is_empty());

    let watch_item = inbox
        .items
        .iter()
        .find(|item| item.symbol.is_some() && item.market_scope.is_some())
        .expect("symbol scoped attention item")
        .clone();
    let actions = vec![
        OwnerAttentionAction {
            action_id: "triage-ack".to_string(),
            item_id: watch_item.item_id.clone(),
            action_type: OwnerAttentionActionType::Acknowledge,
            comment: Some("Paper-only acknowledge".to_string()),
            created_at: Some("2026-05-23T04:30:00Z".to_string()),
            paper_only: true,
        },
        OwnerAttentionAction {
            action_id: "triage-defer".to_string(),
            item_id: watch_item.item_id.clone(),
            action_type: OwnerAttentionActionType::Defer,
            comment: Some("Paper-only defer".to_string()),
            created_at: Some("2026-05-23T04:31:00Z".to_string()),
            paper_only: true,
        },
        OwnerAttentionAction {
            action_id: "triage-dismiss".to_string(),
            item_id: watch_item.item_id.clone(),
            action_type: OwnerAttentionActionType::Dismiss,
            comment: Some("Paper-only dismiss".to_string()),
            created_at: Some("2026-05-23T04:32:00Z".to_string()),
            paper_only: true,
        },
        OwnerAttentionAction {
            action_id: "triage-watch".to_string(),
            item_id: watch_item.item_id.clone(),
            action_type: OwnerAttentionActionType::ConvertToWatchlist,
            comment: Some("Paper-only watchlist candidate".to_string()),
            created_at: Some("2026-05-23T04:33:00Z".to_string()),
            paper_only: true,
        },
        OwnerAttentionAction {
            action_id: "triage-evidence".to_string(),
            item_id: watch_item.item_id.clone(),
            action_type: OwnerAttentionActionType::RequestMoreEvidence,
            comment: Some("Paper-only request more evidence".to_string()),
            created_at: Some("2026-05-23T04:34:00Z".to_string()),
            paper_only: true,
        },
        OwnerAttentionAction {
            action_id: "triage-reconsider".to_string(),
            item_id: watch_item.item_id.clone(),
            action_type: OwnerAttentionActionType::RequestReconsideration,
            comment: Some("Paper-only committee reconsideration".to_string()),
            created_at: Some("2026-05-23T04:35:00Z".to_string()),
            paper_only: true,
        },
        OwnerAttentionAction {
            action_id: "triage-comment".to_string(),
            item_id: watch_item.item_id.clone(),
            action_type: OwnerAttentionActionType::AddComment,
            comment: Some("Paper-only owner comment".to_string()),
            created_at: Some("2026-05-23T04:36:00Z".to_string()),
            paper_only: true,
        },
        OwnerAttentionAction {
            action_id: "triage-unsafe".to_string(),
            item_id: watch_item.item_id.clone(),
            action_type: OwnerAttentionActionType::AddComment,
            comment: Some("execute order with broker account".to_string()),
            created_at: Some("2026-05-23T04:37:00Z".to_string()),
            paper_only: true,
        },
    ];
    let first = run_owner_attention_triage(OwnerAttentionTriageInput {
        previous_run: run.clone(),
        previous_inbox: Some(inbox.clone()),
        owner_actions: actions.clone(),
        watchlist_store: Some(WatchlistCandidateStore::new("triage-test-watchlist")),
    })
    .expect("first triage");
    let second = run_owner_attention_triage(OwnerAttentionTriageInput {
        previous_run: run,
        previous_inbox: Some(inbox),
        owner_actions: actions,
        watchlist_store: Some(WatchlistCandidateStore::new("triage-test-watchlist")),
    })
    .expect("second triage");
    assert_eq!(first, second);

    assert!(first.action_results.iter().any(|result| {
        result.action_id == "triage-ack"
            && result.new_status == OwnerAttentionInboxStatus::Acknowledged
            && result.safety_status == OwnerAttentionActionSafetyStatus::Passed
    }));
    assert!(first.action_results.iter().any(|result| {
        result.action_id == "triage-defer"
            && result.new_status == OwnerAttentionInboxStatus::Deferred
    }));
    assert!(first.action_results.iter().any(|result| {
        result.action_id == "triage-dismiss"
            && result.new_status == OwnerAttentionInboxStatus::Dismissed
    }));
    assert!(first.action_results.iter().any(|result| {
        result.action_id == "triage-unsafe"
            && result.safety_status == OwnerAttentionActionSafetyStatus::Rejected
    }));
    assert!(
        first
            .generated_watchlist_candidates
            .iter()
            .all(|candidate| {
                candidate.paper_only
                    && matches!(
                        candidate.status,
                        WatchlistCandidateStatus::Watching
                            | WatchlistCandidateStatus::NeedsEvidence
                            | WatchlistCandidateStatus::RiskBlocked
                            | WatchlistCandidateStatus::PaperCandidate
                    )
            })
    );
    assert!(first.generated_owner_feedback.iter().any(|feedback| {
        feedback.feedback_type == OwnerFeedbackType::EvidenceRequest && feedback.paper_only
    }));
    assert!(first.generated_owner_feedback.iter().any(|feedback| {
        feedback.feedback_type == OwnerFeedbackType::ReconsiderationRequest && feedback.paper_only
    }));
    assert!(first.generated_owner_feedback.iter().any(|feedback| {
        feedback.feedback_type == OwnerFeedbackType::Comment && feedback.paper_only
    }));
    assert!(first.watchlist_store.active_count >= 1);
    assert!(
        !first
            .watchlist_store
            .candidates_by_symbol(&watch_item.symbol.expect("watch symbol"))
            .is_empty()
    );
    assert!(first.safety_summary.no_broker_order_account);
    assert!(first.safety_summary.no_model_training);
    assert!(first.safety_summary.no_live_inference);

    let inbox_path = std::path::Path::new("target/sprint136_owner_attention_inbox.json");
    let watchlist_path = std::path::Path::new("target/sprint136_watchlist_store.json");
    let _ = std::fs::remove_file(inbox_path);
    let _ = std::fs::remove_file(watchlist_path);
    first
        .inbox
        .save_to_local_json(inbox_path)
        .expect("save inbox");
    let loaded_inbox = OwnerAttentionInbox::load_from_local_json(inbox_path).expect("load inbox");
    assert_eq!(loaded_inbox, first.inbox);
    first
        .watchlist_store
        .save_to_local_json(watchlist_path)
        .expect("save watchlist");
    let loaded_watchlist =
        WatchlistCandidateStore::load_from_local_json(watchlist_path).expect("load watchlist");
    assert_eq!(loaded_watchlist, first.watchlist_store);
    let _ = std::fs::remove_file(inbox_path);
    let _ = std::fs::remove_file(watchlist_path);

    let err = OwnerAttentionInbox::load_from_local_json(std::path::Path::new(
        "https://example.invalid/inbox.json",
    ))
    .expect_err("remote inbox path must fail");
    assert!(err.contains("must be local"));
    let err = WatchlistCandidateStore::load_from_local_json(std::path::Path::new(
        "https://example.invalid/watchlist.json",
    ))
    .expect_err("remote watchlist path must fail");
    assert!(err.contains("must be local"));
    let sample_actions = load_owner_attention_actions_from_local_json(std::path::Path::new(
        "examples/minimal_owner_attention_actions.sample.json",
    ))
    .expect("load owner attention action sample");
    assert!(!sample_actions.is_empty());
}

#[test]
fn owner_daily_brief_store_appends_loads_and_rejects_remote_paths() {
    let run = run_autonomous_paper_committee_loop_from_config_path(std::path::Path::new(
        "examples/soma_minimal_ai_committee_core.toml",
    ))
    .expect("autonomous run with daily brief");
    let brief = run
        .watchlist_recheck
        .as_ref()
        .and_then(|recheck| recheck.owner_daily_brief.clone())
        .expect("owner daily brief");
    let mut store = OwnerDailyBriefStore::new("owner-daily-brief-store-test");
    store.append_brief(brief.clone());
    assert_eq!(
        store.latest().map(|item| &item.brief_id),
        Some(&brief.brief_id)
    );
    let mut earlier_sorting_brief = brief.clone();
    earlier_sorting_brief.brief_id = "brief-0000-appended-last".to_string();
    store.append_brief(earlier_sorting_brief.clone());
    assert_eq!(
        store.latest().map(|item| &item.brief_id),
        Some(&earlier_sorting_brief.brief_id)
    );
    assert!(
        !store
            .briefs_by_symbol(&brief.reviewed_symbols[0])
            .is_empty()
    );
    assert_eq!(
        store.risk_veto_count(),
        brief.risk_vetoes.len() + earlier_sorting_brief.risk_vetoes.len()
    );
    assert_eq!(
        store.need_more_evidence_count(),
        brief.need_more_evidence_items.len() + earlier_sorting_brief.need_more_evidence_items.len()
    );

    let path = std::path::Path::new("target/sprint141_owner_daily_brief_store.json");
    let _ = std::fs::remove_file(path);
    store.save_to_local_json(path).expect("save brief store");
    let loaded = OwnerDailyBriefStore::load_from_local_json(path).expect("load brief store");
    assert_eq!(loaded, store);
    let _ = std::fs::remove_file(path);

    let err = OwnerDailyBriefStore::load_from_local_json(std::path::Path::new(
        "https://example.invalid/brief-store.json",
    ))
    .expect_err("remote brief store path must fail");
    assert!(err.contains("must be local"));

    let mut unsafe_store = OwnerDailyBriefStore::new("unsafe-owner-daily-brief-store-test");
    let mut unsafe_brief = brief;
    unsafe_brief
        .next_owner_attention
        .push("execute order through broker account".to_string());
    unsafe_store.append_brief(unsafe_brief);
    let err = unsafe_store
        .save_to_local_json(path)
        .expect_err("unsafe brief store must fail");
    assert!(err.contains("unsafe instruction"));
}

#[test]
fn committee_state_snapshot_exports_ui_shape_safely_and_deterministically() {
    let run = run_autonomous_paper_committee_loop_from_config_path(std::path::Path::new(
        "examples/soma_minimal_ai_committee_core.toml",
    ))
    .expect("autonomous run with snapshot");
    let triage = run
        .owner_attention_triage
        .as_ref()
        .expect("owner attention triage");
    let recheck = run.watchlist_recheck.as_ref().expect("watchlist recheck");
    let brief = recheck
        .owner_daily_brief
        .clone()
        .expect("owner daily brief");
    let mut brief_store = OwnerDailyBriefStore::new("snapshot-brief-store");
    brief_store.append_brief(brief);
    let input = CommitteeStateExportInput {
        autonomous_run_result: Some(run.clone()),
        watchlist_recheck_result: run.watchlist_recheck.clone(),
        attention_inbox: Some(triage.inbox.clone()),
        watchlist_store: Some(recheck.updated_watchlist_store.clone()),
        member_state_store: Some(MemberStateStore {
            store_id: "snapshot-member-state-store".to_string(),
            members: run.final_member_states.clone(),
            source_label: "unit-test".to_string(),
            paper_only: true,
        }),
        owner_daily_brief_store: Some(brief_store),
        owner_summary: Some(recheck.owner_summary.clone()),
        owner_console_view: recheck.owner_console_view.clone(),
    };
    let first = build_committee_state_snapshot(input.clone()).expect("first snapshot");
    let second = build_committee_state_snapshot(input).expect("second snapshot");
    assert_eq!(
        serde_json::to_string_pretty(&first).expect("first json"),
        serde_json::to_string_pretty(&second).expect("second json")
    );
    assert!(first.paper_only);
    assert!(first.safety_summary.no_broker_order_account);
    assert!(first.safety_summary.no_model_training);
    assert!(first.safety_summary.no_live_inference);
    assert_eq!(first.members.len(), 3);
    assert!(!first.watchlist_candidates.is_empty());
    assert!(!first.attention_items.is_empty());
    assert!(!first.recent_chairman_decisions.is_empty());
    assert!(!first.recent_risk_vetoes.is_empty());
    assert!(!first.next_owner_actions.is_empty());
    assert!(
        first
            .next_owner_actions
            .iter()
            .any(|row| row.action_type == NextOwnerActionType::ReviewRiskVeto)
    );
    assert!(first.latest_owner_daily_brief.is_some());
    let json = serde_json::to_string(&first).expect("snapshot json");
    for forbidden_field in ["\"broker\"", "\"order\"", "\"account\""] {
        assert!(!json.contains(forbidden_field));
    }

    let path = std::path::Path::new("target/sprint141_committee_state_snapshot.json");
    let _ = std::fs::remove_file(path);
    save_committee_state_snapshot(path, &first).expect("save snapshot");
    assert!(path.exists());
    let _ = std::fs::remove_file(path);
    let err = save_committee_state_snapshot(
        std::path::Path::new("https://example.invalid/snapshot.json"),
        &first,
    )
    .expect_err("remote snapshot path must fail");
    assert!(err.contains("must be local"));
}

#[test]
fn committee_state_export_writes_state_contract_and_owner_read_model() {
    let run = run_autonomous_paper_committee_loop_from_config_path(std::path::Path::new(
        "examples/soma_minimal_ai_committee_core.toml",
    ))
    .expect("autonomous run with snapshot");
    let snapshot = run
        .committee_state_snapshot
        .clone()
        .expect("committee state snapshot");
    let root = std::path::Path::new("target/sprint142_state_export_contract");
    let _ = std::fs::remove_dir_all(root);
    let result = write_committee_state_export(
        CommitteeStateExportConfig {
            export_id: "sprint142-state-export".to_string(),
            export_root_path: root.display().to_string(),
            write_latest_snapshot: true,
            write_history_snapshot: true,
            write_snapshot_index: true,
            write_owner_console_read_model: true,
            schema_version: "committee-state.v1".to_string(),
            max_history_entries: Some(1),
            paper_only: true,
        },
        &snapshot,
        CommitteeStateSnapshotSource::ManualExport,
    )
    .expect("state export writes");
    assert!(result.wrote_latest_snapshot);
    assert!(result.wrote_history_snapshot);
    assert!(result.wrote_snapshot_index);
    assert!(result.wrote_owner_console_read_model);
    assert!(root.join("latest.json").exists());
    assert!(root.join("snapshot_index.json").exists());
    assert!(root.join("owner_console_read_model.json").exists());
    assert!(
        root.join("history")
            .join(format!("{}.json", snapshot.snapshot_id))
            .exists()
    );
    let policy = SnapshotFileNamingPolicy::state_folder_contract();
    assert_eq!(policy.latest_path(root), root.join("latest.json"));
    assert_eq!(
        policy.snapshot_index_path(root),
        root.join("snapshot_index.json")
    );
    assert_eq!(
        policy
            .history_path(root, &snapshot.snapshot_id)
            .expect("history path"),
        root.join("history")
            .join(format!("{}.json", snapshot.snapshot_id))
    );
    assert_eq!(
        policy.owner_console_read_model_path(root),
        root.join("owner_console_read_model.json")
    );

    let deterministic_latest = std::fs::read_to_string(root.join("latest.json")).expect("latest");
    write_committee_state_export(
        CommitteeStateExportConfig {
            export_id: "sprint142-state-export-repeat".to_string(),
            export_root_path: root.display().to_string(),
            write_latest_snapshot: true,
            write_history_snapshot: true,
            write_snapshot_index: true,
            write_owner_console_read_model: true,
            schema_version: "committee-state.v1".to_string(),
            max_history_entries: Some(1),
            paper_only: true,
        },
        &snapshot,
        CommitteeStateSnapshotSource::ManualExport,
    )
    .expect("repeated state export writes");
    assert_eq!(
        deterministic_latest,
        std::fs::read_to_string(root.join("latest.json")).expect("repeated latest")
    );

    let latest: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(root.join("latest.json")).expect("latest json"),
    )
    .expect("latest envelope");
    assert_eq!(latest["schema_version"], "committee-state.v1");
    assert_eq!(latest["snapshot_id"], snapshot.snapshot_id);
    assert_eq!(latest["source"], "manual_export");
    assert_eq!(latest["safety"]["paper_only"], true);
    assert_eq!(latest["safety"]["no_live_trading"], true);

    let read_model = build_owner_console_read_model("committee-state.v1", &snapshot);
    assert_eq!(read_model.snapshot_id, snapshot.snapshot_id);
    assert!(!read_model.member_cards.is_empty());
    assert!(!read_model.watchlist_cards.is_empty());
    assert!(!read_model.attention_cards.is_empty());
    assert!(!read_model.decision_cards.is_empty());
    assert!(!read_model.next_action_cards.is_empty());
    assert!(
        read_model
            .safety_badges
            .iter()
            .any(|badge| badge == "read-only")
    );
    let read_model_json: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(root.join("owner_console_read_model.json"))
            .expect("read model json"),
    )
    .expect("owner console read model");
    assert_eq!(read_model_json["schema_version"], "committee-state.v1");
    assert!(
        read_model_json["header"]["reviewed_symbol_count"]
            .as_u64()
            .expect("reviewed symbol count")
            > 0
    );

    let mut next_snapshot = snapshot.clone();
    next_snapshot.snapshot_id = format!("{}-next", snapshot.snapshot_id);
    next_snapshot.generated_at = Some("zzzz-next-snapshot".to_string());
    write_committee_state_export(
        CommitteeStateExportConfig {
            export_id: "sprint142-state-export-second".to_string(),
            export_root_path: root.display().to_string(),
            write_latest_snapshot: false,
            write_history_snapshot: true,
            write_snapshot_index: true,
            write_owner_console_read_model: false,
            schema_version: "committee-state.v1".to_string(),
            max_history_entries: Some(1),
            paper_only: true,
        },
        &next_snapshot,
        CommitteeStateSnapshotSource::ManualExport,
    )
    .expect("second state export writes");
    let index: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(root.join("snapshot_index.json")).expect("index json"),
    )
    .expect("snapshot index");
    assert_eq!(index["latest_snapshot_id"], next_snapshot.snapshot_id);
    assert_eq!(index["entries"].as_array().expect("entries").len(), 1);
    assert_eq!(
        index["entries"][0]["path"],
        format!("history/{}.json", next_snapshot.snapshot_id)
    );
    let mut older_latest_snapshot = snapshot.clone();
    older_latest_snapshot.snapshot_id = format!("{}-older-latest", snapshot.snapshot_id);
    older_latest_snapshot.generated_at = Some("0000-older-latest".to_string());
    write_committee_state_export(
        CommitteeStateExportConfig {
            export_id: "sprint142-state-export-older-latest".to_string(),
            export_root_path: root.display().to_string(),
            write_latest_snapshot: false,
            write_history_snapshot: true,
            write_snapshot_index: true,
            write_owner_console_read_model: false,
            schema_version: "committee-state.v1".to_string(),
            max_history_entries: Some(1),
            paper_only: true,
        },
        &older_latest_snapshot,
        CommitteeStateSnapshotSource::ManualExport,
    )
    .expect("older latest state export writes");
    let capped_index: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(root.join("snapshot_index.json")).expect("capped index json"),
    )
    .expect("capped snapshot index");
    assert_eq!(
        capped_index["latest_snapshot_id"],
        older_latest_snapshot.snapshot_id
    );
    assert_eq!(
        capped_index["entries"]
            .as_array()
            .expect("capped entries")
            .len(),
        1
    );
    assert_eq!(
        capped_index["entries"][0]["snapshot_id"],
        older_latest_snapshot.snapshot_id
    );
    let export_text =
        std::fs::read_to_string(root.join("latest.json")).expect("safe latest export text");
    for forbidden_field in ["\"broker\"", "\"order\"", "\"account\""] {
        assert!(!export_text.contains(forbidden_field));
    }

    let remote_err = write_committee_state_export(
        CommitteeStateExportConfig {
            export_id: "sprint142-state-export-remote".to_string(),
            export_root_path: "https://example.invalid/state".to_string(),
            write_latest_snapshot: true,
            write_history_snapshot: false,
            write_snapshot_index: false,
            write_owner_console_read_model: false,
            schema_version: "committee-state.v1".to_string(),
            max_history_entries: None,
            paper_only: true,
        },
        &snapshot,
        CommitteeStateSnapshotSource::ManualExport,
    )
    .expect_err("remote export path must fail");
    assert!(remote_err.contains("local path"));
    let traversal_err = write_committee_state_export(
        CommitteeStateExportConfig {
            export_id: "sprint142-state-export-traversal".to_string(),
            export_root_path: "target/../sprint142_bad".to_string(),
            write_latest_snapshot: true,
            write_history_snapshot: false,
            write_snapshot_index: false,
            write_owner_console_read_model: false,
            schema_version: "committee-state.v1".to_string(),
            max_history_entries: None,
            paper_only: true,
        },
        &snapshot,
        CommitteeStateSnapshotSource::ManualExport,
    )
    .expect_err("parent traversal must fail");
    assert!(traversal_err.contains("parent-directory traversal"));
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn owner_console_viewer_loads_and_renders_read_only_terminal_view() {
    let run = run_autonomous_paper_committee_loop_from_config_path(std::path::Path::new(
        "examples/soma_minimal_ai_committee_core.toml",
    ))
    .expect("autonomous run with owner read model");
    let snapshot = run
        .committee_state_snapshot
        .clone()
        .expect("committee state snapshot");
    let root = std::path::Path::new("target/sprint143_owner_console_viewer_state");
    let _ = std::fs::remove_dir_all(root);
    write_committee_state_export(
        CommitteeStateExportConfig {
            export_id: "sprint143-owner-console-viewer".to_string(),
            export_root_path: root.display().to_string(),
            write_latest_snapshot: false,
            write_history_snapshot: false,
            write_snapshot_index: false,
            write_owner_console_read_model: true,
            schema_version: "committee-state.v1".to_string(),
            max_history_entries: None,
            paper_only: true,
        },
        &snapshot,
        CommitteeStateSnapshotSource::ManualExport,
    )
    .expect("write read model");
    let read_model_path = root.join("owner_console_read_model.json");
    let before = std::fs::read_to_string(&read_model_path).expect("read model before viewer");
    let (source, read_model) = load_owner_console_read_model_from_local_file(&read_model_path)
        .expect("load local read model");
    assert_eq!(source.schema_version, "committee-state.v1");
    assert_eq!(
        source.snapshot_id.as_deref(),
        Some(snapshot.snapshot_id.as_str())
    );

    let options = OwnerConsoleTerminalOptions {
        max_width: Some(160),
        ..OwnerConsoleTerminalOptions::default()
    };
    let terminal_view = render_owner_console_terminal_view(&read_model, &options);
    assert!(terminal_view.title.contains("Soma Owner Console"));
    assert!(
        terminal_view
            .header_lines
            .iter()
            .any(|line| line.contains("reviewed_symbols="))
    );
    assert_eq!(terminal_view.member_sections.len(), 3);
    assert!(!terminal_view.watchlist_sections.is_empty());
    assert!(!terminal_view.attention_sections.is_empty());
    assert!(!terminal_view.decision_sections.is_empty());
    assert!(!terminal_view.next_action_sections.is_empty());
    assert!(
        terminal_view
            .safety_lines
            .iter()
            .any(|line| line.contains("paper-only"))
    );

    let first = run_owner_console_viewer(&read_model_path, options.clone()).expect("viewer result");
    let second = run_owner_console_viewer(&read_model_path, options).expect("repeat viewer result");
    assert_eq!(first.rendered_text, second.rendered_text);
    assert!(first.rendered_text.contains("Header"));
    assert!(first.rendered_text.contains("Members"));
    assert!(first.rendered_text.contains("Watchlist"));
    assert!(first.rendered_text.contains("Attention"));
    assert!(first.rendered_text.contains("Decisions"));
    assert!(first.rendered_text.contains("Next Actions"));
    assert!(first.rendered_text.contains("Safety"));
    assert!(first.safety_status.contains("read-only"));
    let rendered_json = serde_json::to_string(&first.terminal_view).expect("terminal view json");
    for forbidden_field in ["\"broker\"", "\"order\"", "\"account\""] {
        assert!(!rendered_json.contains(forbidden_field));
    }
    let after = std::fs::read_to_string(&read_model_path).expect("read model after viewer");
    assert_eq!(before, after);

    let remote_err = load_owner_console_read_model_from_local_file(std::path::Path::new(
        "https://example.invalid/owner_console_read_model.json",
    ))
    .expect_err("remote read model path must fail");
    assert!(remote_err.contains("local"));
    let traversal_err = load_owner_console_read_model_from_local_file(std::path::Path::new(
        "target/../owner_console_read_model.json",
    ))
    .expect_err("path traversal must fail");
    assert!(traversal_err.contains("parent-directory traversal"));
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn owner_action_composer_writes_safe_deterministic_action_file() {
    let run = run_autonomous_paper_committee_loop_from_config_path(std::path::Path::new(
        "examples/soma_minimal_ai_committee_core.toml",
    ))
    .expect("autonomous run with owner read model");
    let snapshot = run
        .committee_state_snapshot
        .clone()
        .expect("committee state snapshot");
    let root = std::path::Path::new("target/sprint144_owner_action_composer");
    let _ = std::fs::remove_dir_all(root);
    write_committee_state_export(
        CommitteeStateExportConfig {
            export_id: "sprint144-owner-action-composer".to_string(),
            export_root_path: root.display().to_string(),
            write_latest_snapshot: false,
            write_history_snapshot: false,
            write_snapshot_index: false,
            write_owner_console_read_model: true,
            schema_version: "committee-state.v1".to_string(),
            max_history_entries: None,
            paper_only: true,
        },
        &snapshot,
        CommitteeStateSnapshotSource::ManualExport,
    )
    .expect("write read model");
    let read_model_path = root.join("owner_console_read_model.json");
    let output_path = root.join("owner_attention_actions.json");
    let (_source, read_model) =
        load_owner_console_read_model_from_local_file(&read_model_path).expect("load read model");
    let target_item_id = read_model
        .attention_cards
        .first()
        .map(|card| card.item_id.clone())
        .expect("attention card target");
    let before = std::fs::read_to_string(&read_model_path).expect("state before compose");
    let dry_run = compose_owner_action_from_read_model(OwnerActionComposerConfig {
        read_model_path: read_model_path.display().to_string(),
        output_actions_path: output_path.display().to_string(),
        target_item_id: Some(target_item_id.clone()),
        action_type: OwnerAttentionActionType::RequestMoreEvidence,
        comment: Some("근거가 부족하니 다시 검토".to_string()),
        dry_run: true,
        paper_only: true,
    })
    .expect("dry-run compose");
    assert!(dry_run.target_item_found);
    assert!(!dry_run.wrote_output);
    assert!(!output_path.exists());
    assert!(dry_run.preview_text.contains("RequestMoreEvidence"));

    let write_result = compose_owner_action_from_read_model(OwnerActionComposerConfig {
        read_model_path: read_model_path.display().to_string(),
        output_actions_path: output_path.display().to_string(),
        target_item_id: Some(target_item_id.clone()),
        action_type: OwnerAttentionActionType::RequestMoreEvidence,
        comment: Some("근거가 부족하니 다시 검토".to_string()),
        dry_run: false,
        paper_only: true,
    })
    .expect("write compose");
    assert!(write_result.wrote_output);
    assert!(output_path.exists());
    let first_text = std::fs::read_to_string(&output_path).expect("action file");
    let action_file: OwnerActionFile = serde_json::from_str(&first_text).expect("action file json");
    assert!(action_file.paper_only);
    assert_eq!(action_file.actions.len(), 1);
    assert_eq!(action_file.actions[0].item_id, target_item_id);
    assert_eq!(
        action_file.actions[0].action_type,
        OwnerAttentionActionType::RequestMoreEvidence
    );
    let loaded_actions =
        load_owner_attention_actions_from_local_json(&output_path).expect("load action file");
    assert_eq!(loaded_actions, action_file.actions);
    for forbidden_field in ["\"broker\"", "\"order\"", "\"account\""] {
        assert!(!first_text.contains(forbidden_field));
    }

    compose_owner_action_from_read_model(OwnerActionComposerConfig {
        read_model_path: read_model_path.display().to_string(),
        output_actions_path: output_path.display().to_string(),
        target_item_id: Some(target_item_id.clone()),
        action_type: OwnerAttentionActionType::RequestMoreEvidence,
        comment: Some("근거가 부족하니 다시 검토".to_string()),
        dry_run: false,
        paper_only: true,
    })
    .expect("repeat compose");
    assert_eq!(
        first_text,
        std::fs::read_to_string(&output_path).expect("repeat action file")
    );
    assert_eq!(
        before,
        std::fs::read_to_string(&read_model_path).expect("state after compose")
    );

    let general_comment_path = root.join("owner_general_comment_actions.json");
    let general_comment = compose_owner_action_from_read_model(OwnerActionComposerConfig {
        read_model_path: read_model_path.display().to_string(),
        output_actions_path: general_comment_path.display().to_string(),
        target_item_id: None,
        action_type: OwnerAttentionActionType::AddComment,
        comment: Some("확인함".to_string()),
        dry_run: false,
        paper_only: true,
    })
    .expect("targetless add-comment compose");
    assert!(!general_comment.target_item_found);
    assert!(general_comment.wrote_output);
    let general_actions = load_owner_attention_actions_from_local_json(&general_comment_path)
        .expect("load targetless add-comment action");
    assert_eq!(general_actions.len(), 1);
    assert_eq!(general_actions[0].item_id, "owner-general-comment");
    assert_eq!(
        general_actions[0].action_type,
        OwnerAttentionActionType::AddComment
    );

    let unknown_err = compose_owner_action_from_read_model(OwnerActionComposerConfig {
        read_model_path: read_model_path.display().to_string(),
        output_actions_path: output_path.display().to_string(),
        target_item_id: Some("missing-attention-item".to_string()),
        action_type: OwnerAttentionActionType::RequestMoreEvidence,
        comment: Some("근거가 부족하니 다시 검토".to_string()),
        dry_run: true,
        paper_only: true,
    })
    .expect_err("unknown target rejected");
    assert!(unknown_err.contains("not found"));
    let remote_state_err = compose_owner_action_from_read_model(OwnerActionComposerConfig {
        read_model_path: "https://example.invalid/owner_console_read_model.json".to_string(),
        output_actions_path: output_path.display().to_string(),
        target_item_id: Some(target_item_id.clone()),
        action_type: OwnerAttentionActionType::RequestMoreEvidence,
        comment: Some("근거가 부족하니 다시 검토".to_string()),
        dry_run: true,
        paper_only: true,
    })
    .expect_err("remote state rejected");
    assert!(remote_state_err.contains("local"));
    let remote_output_err = compose_owner_action_from_read_model(OwnerActionComposerConfig {
        read_model_path: read_model_path.display().to_string(),
        output_actions_path: "https://example.invalid/actions.json".to_string(),
        target_item_id: Some(target_item_id.clone()),
        action_type: OwnerAttentionActionType::RequestMoreEvidence,
        comment: Some("근거가 부족하니 다시 검토".to_string()),
        dry_run: true,
        paper_only: true,
    })
    .expect_err("remote output rejected");
    assert!(remote_output_err.contains("local"));
    let traversal_err = compose_owner_action_from_read_model(OwnerActionComposerConfig {
        read_model_path: read_model_path.display().to_string(),
        output_actions_path: "target/../owner_attention_actions.json".to_string(),
        target_item_id: Some(target_item_id),
        action_type: OwnerAttentionActionType::RequestMoreEvidence,
        comment: Some("근거가 부족하니 다시 검토".to_string()),
        dry_run: true,
        paper_only: true,
    })
    .expect_err("path traversal rejected");
    assert!(traversal_err.contains("parent-directory traversal"));
    let unsafe_order_err = compose_owner_action_from_read_model(OwnerActionComposerConfig {
        read_model_path: read_model_path.display().to_string(),
        output_actions_path: output_path.display().to_string(),
        target_item_id: read_model
            .attention_cards
            .first()
            .map(|card| card.item_id.clone()),
        action_type: OwnerAttentionActionType::RequestMoreEvidence,
        comment: Some("계좌에서 주문 넣어".to_string()),
        dry_run: true,
        paper_only: true,
    })
    .expect_err("unsafe order text rejected");
    assert!(unsafe_order_err.contains("unsafe instruction"));
    let unsafe_leverage_err = compose_owner_action_from_read_model(OwnerActionComposerConfig {
        read_model_path: read_model_path.display().to_string(),
        output_actions_path: output_path.display().to_string(),
        target_item_id: read_model
            .attention_cards
            .first()
            .map(|card| card.item_id.clone()),
        action_type: OwnerAttentionActionType::RequestMoreEvidence,
        comment: Some("레버리지 최대로".to_string()),
        dry_run: true,
        paper_only: true,
    })
    .expect_err("unsafe leverage text rejected");
    assert!(unsafe_leverage_err.contains("unsafe instruction"));
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn owner_action_composer_outputs_feed_existing_triage_actions() {
    let run = run_autonomous_paper_committee_loop_from_config_path(std::path::Path::new(
        "examples/soma_minimal_ai_committee_core.toml",
    ))
    .expect("autonomous run with owner attention");
    let snapshot = run
        .committee_state_snapshot
        .clone()
        .expect("committee state snapshot");
    let root = std::path::Path::new("target/sprint144_owner_action_triage_bridge");
    let _ = std::fs::remove_dir_all(root);
    write_committee_state_export(
        CommitteeStateExportConfig {
            export_id: "sprint144-owner-action-triage".to_string(),
            export_root_path: root.display().to_string(),
            write_latest_snapshot: false,
            write_history_snapshot: false,
            write_snapshot_index: false,
            write_owner_console_read_model: true,
            schema_version: "committee-state.v1".to_string(),
            max_history_entries: None,
            paper_only: true,
        },
        &snapshot,
        CommitteeStateSnapshotSource::ManualExport,
    )
    .expect("write read model");
    let read_model_path = root.join("owner_console_read_model.json");
    let (_source, read_model) =
        load_owner_console_read_model_from_local_file(&read_model_path).expect("load read model");
    let attention_ids = read_model
        .attention_cards
        .iter()
        .map(|card| card.item_id.clone())
        .collect::<Vec<_>>();
    assert!(attention_ids.len() >= 4);
    let watch_target_id = run
        .attention_queue
        .items
        .iter()
        .find(|item| {
            item.symbol.is_some()
                && item.market_scope.is_some()
                && attention_ids.iter().any(|id| id == &item.item_id)
        })
        .map(|item| item.item_id.clone())
        .expect("watchlist-convertible attention item");
    let action_specs = [
        (
            OwnerAttentionActionType::RequestMoreEvidence,
            Some(attention_ids[0].clone()),
            "근거가 부족하니 다시 검토",
        ),
        (
            OwnerAttentionActionType::RequestReconsideration,
            Some(attention_ids[1].clone()),
            "리스크 위원 의견을 다시 반영",
        ),
        (
            OwnerAttentionActionType::ConvertToWatchlist,
            Some(watch_target_id),
            "관심종목으로 보류",
        ),
        (
            OwnerAttentionActionType::AddComment,
            Some(attention_ids[3].clone()),
            "확인함",
        ),
        (OwnerAttentionActionType::AddComment, None, "확인함"),
    ];
    let mut owner_actions = Vec::new();
    for (index, (action_type, item_id, comment)) in action_specs.iter().enumerate() {
        let path = root.join(format!("owner_attention_actions_{index}.json"));
        compose_owner_action_from_read_model(OwnerActionComposerConfig {
            read_model_path: read_model_path.display().to_string(),
            output_actions_path: path.display().to_string(),
            target_item_id: item_id.clone(),
            action_type: *action_type,
            comment: Some((*comment).to_string()),
            dry_run: false,
            paper_only: true,
        })
        .expect("compose triage action");
        owner_actions.extend(
            load_owner_attention_actions_from_local_json(&path).expect("load composed action"),
        );
    }
    let triage = run_owner_attention_triage(OwnerAttentionTriageInput {
        previous_run: run,
        previous_inbox: None,
        owner_actions,
        watchlist_store: None,
    })
    .expect("triage consumes composed actions");
    assert!(
        triage
            .action_results
            .iter()
            .all(|result| result.safety_status == OwnerAttentionActionSafetyStatus::Passed)
    );
    assert!(triage.generated_owner_feedback_count >= 4);
    assert!(!triage.generated_watchlist_candidates.is_empty());
    assert!(
        triage
            .generated_owner_feedback
            .iter()
            .any(|feedback| feedback.feedback_type == OwnerFeedbackType::EvidenceRequest)
    );
    assert!(
        triage
            .generated_owner_feedback
            .iter()
            .any(|feedback| feedback.feedback_type == OwnerFeedbackType::ReconsiderationRequest)
    );
    assert!(
        triage
            .generated_owner_feedback
            .iter()
            .any(|feedback| feedback.feedback_type == OwnerFeedbackType::Comment)
    );
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn owner_action_consumption_applies_ledgers_duplicates_and_refreshes_state() {
    let run = run_autonomous_paper_committee_loop_from_config_path(std::path::Path::new(
        "examples/soma_minimal_ai_committee_core.toml",
    ))
    .expect("autonomous run for owner action consumption");
    let snapshot = run
        .committee_state_snapshot
        .clone()
        .expect("committee state snapshot");
    let root = std::path::Path::new("target/sprint145_owner_action_consumption");
    let _ = std::fs::remove_dir_all(root);
    write_committee_state_export(
        CommitteeStateExportConfig {
            export_id: "sprint145-owner-action-consumption".to_string(),
            export_root_path: root.display().to_string(),
            write_latest_snapshot: true,
            write_history_snapshot: true,
            write_snapshot_index: true,
            write_owner_console_read_model: true,
            schema_version: "committee-state.v1".to_string(),
            max_history_entries: Some(3),
            paper_only: true,
        },
        &snapshot,
        CommitteeStateSnapshotSource::ManualExport,
    )
    .expect("write read model");
    let read_model_path = root.join("owner_console_read_model.json");
    let action_path = root.join("owner_attention_actions.json");
    let ledger_path = root.join("owner_action_processing_ledger.json");
    let inbox_path = root.join("owner_attention_inbox.json");
    let watchlist_path = root.join("owner_watchlist_store.json");
    let member_state_path = root.join("member_state_store.json");
    let (_source, read_model) =
        load_owner_console_read_model_from_local_file(&read_model_path).expect("load read model");
    let target_item_id = read_model
        .attention_cards
        .first()
        .map(|card| card.item_id.clone())
        .expect("attention target");
    compose_owner_action_from_read_model(OwnerActionComposerConfig {
        read_model_path: read_model_path.display().to_string(),
        output_actions_path: action_path.display().to_string(),
        target_item_id: Some(target_item_id),
        action_type: OwnerAttentionActionType::RequestMoreEvidence,
        comment: Some("근거가 부족하니 다시 검토".to_string()),
        dry_run: false,
        paper_only: true,
    })
    .expect("compose owner action");

    let dry_run = consume_owner_action_file_with_previous_run(
        OwnerActionConsumptionConfig {
            action_file_path: action_path.display().to_string(),
            processed_action_ledger_path: Some(ledger_path.display().to_string()),
            inbox_input_path: None,
            inbox_output_path: Some(inbox_path.display().to_string()),
            watchlist_input_path: None,
            watchlist_output_path: Some(watchlist_path.display().to_string()),
            member_state_input_path: None,
            member_state_output_path: Some(member_state_path.display().to_string()),
            committee_state_export_root_path: Some(root.display().to_string()),
            owner_console_read_model_path: Some(read_model_path.display().to_string()),
            apply_mode: OwnerActionApplyMode::DryRun,
            duplicate_policy: OwnerActionDuplicatePolicy::SkipAlreadyProcessed,
            after_apply_file_policy: OwnerActionAfterApplyFilePolicy::KeepActionFile,
            paper_only: true,
        },
        Some(run.clone()),
    )
    .expect("dry-run consumption");
    assert!(dry_run.dry_run);
    assert_eq!(dry_run.applied_action_count, 0);
    assert!(!ledger_path.exists());
    assert!(!inbox_path.exists());

    let apply_config = OwnerActionConsumptionConfig {
        action_file_path: action_path.display().to_string(),
        processed_action_ledger_path: Some(ledger_path.display().to_string()),
        inbox_input_path: None,
        inbox_output_path: Some(inbox_path.display().to_string()),
        watchlist_input_path: None,
        watchlist_output_path: Some(watchlist_path.display().to_string()),
        member_state_input_path: None,
        member_state_output_path: Some(member_state_path.display().to_string()),
        committee_state_export_root_path: Some(root.display().to_string()),
        owner_console_read_model_path: Some(read_model_path.display().to_string()),
        apply_mode: OwnerActionApplyMode::Apply,
        duplicate_policy: OwnerActionDuplicatePolicy::SkipAlreadyProcessed,
        after_apply_file_policy: OwnerActionAfterApplyFilePolicy::KeepActionFile,
        paper_only: true,
    };
    let applied =
        consume_owner_action_file_with_previous_run(apply_config.clone(), Some(run.clone()))
            .expect("apply consumption");
    assert_eq!(applied.loaded_action_count, 1);
    assert_eq!(applied.applied_action_count, 1);
    assert_eq!(applied.rejected_action_count, 0);
    assert!(ledger_path.exists());
    assert!(inbox_path.exists());
    assert!(member_state_path.exists());
    assert!(
        applied
            .generated_owner_feedback
            .iter()
            .any(|feedback| feedback.feedback_type == OwnerFeedbackType::EvidenceRequest)
    );
    assert!(
        applied
            .reconsideration_results
            .iter()
            .any(|result| result.reconsideration_session_count > 0)
    );
    assert!(applied.updated_committee_state_snapshot.is_some());
    assert!(applied.state_export_result.is_some());
    assert!(
        applied
            .processing_ledger
            .processed_actions
            .iter()
            .any(|record| record.processed_status == OwnerActionProcessedStatus::Applied)
    );

    let skipped = consume_owner_action_file_with_previous_run(apply_config.clone(), Some(run))
        .expect("duplicate skip");
    assert_eq!(skipped.applied_action_count, 0);
    assert_eq!(skipped.skipped_duplicate_count, 1);
    let duplicate_reject = consume_owner_action_file(OwnerActionConsumptionConfig {
        duplicate_policy: OwnerActionDuplicatePolicy::RejectAlreadyProcessed,
        ..apply_config
    })
    .expect_err("duplicate reject");
    assert!(duplicate_reject.contains("already processed"));
    let refreshed_read_model =
        std::fs::read_to_string(&read_model_path).expect("refreshed read model exists");
    assert!(refreshed_read_model.contains("committee-state.v1"));
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn owner_action_consumption_updates_inbox_watchlist_and_general_comment() {
    let run = run_autonomous_paper_committee_loop_from_config_path(std::path::Path::new(
        "examples/soma_minimal_ai_committee_core.toml",
    ))
    .expect("autonomous run for action behavior");
    let root = std::path::Path::new("target/sprint145_owner_action_behavior");
    let _ = std::fs::remove_dir_all(root);
    std::fs::create_dir_all(root).expect("create behavior root");
    let inbox_items = OwnerAttentionInbox::from_attention_queue(&run.attention_queue).items;
    let watch_item = inbox_items
        .iter()
        .find(|item| item.symbol.is_some() && item.market_scope.is_some())
        .expect("watchlist convertible item");
    let distinct_item_ids = inbox_items
        .iter()
        .map(|item| item.item_id.clone())
        .filter(|item_id| item_id != &watch_item.item_id)
        .fold(Vec::<String>::new(), |mut ids, item_id| {
            if !ids.contains(&item_id) {
                ids.push(item_id);
            }
            ids
        });
    assert!(distinct_item_ids.len() >= 4);
    let actions = vec![
        OwnerAttentionAction {
            action_id: "sprint145-ack".to_string(),
            item_id: distinct_item_ids[0].clone(),
            action_type: OwnerAttentionActionType::Acknowledge,
            comment: Some("확인함".to_string()),
            created_at: None,
            paper_only: true,
        },
        OwnerAttentionAction {
            action_id: "sprint145-defer".to_string(),
            item_id: distinct_item_ids[1].clone(),
            action_type: OwnerAttentionActionType::Defer,
            comment: Some("관심종목으로 보류".to_string()),
            created_at: None,
            paper_only: true,
        },
        OwnerAttentionAction {
            action_id: "sprint145-dismiss".to_string(),
            item_id: distinct_item_ids[2].clone(),
            action_type: OwnerAttentionActionType::Dismiss,
            comment: Some("이 항목은 dismiss".to_string()),
            created_at: None,
            paper_only: true,
        },
        OwnerAttentionAction {
            action_id: "sprint145-watch".to_string(),
            item_id: watch_item.item_id.clone(),
            action_type: OwnerAttentionActionType::ConvertToWatchlist,
            comment: Some("관심종목으로 보류".to_string()),
            created_at: None,
            paper_only: true,
        },
        OwnerAttentionAction {
            action_id: "sprint145-reconsider".to_string(),
            item_id: distinct_item_ids[3].clone(),
            action_type: OwnerAttentionActionType::RequestReconsideration,
            comment: Some("리스크 위원 의견을 다시 반영".to_string()),
            created_at: None,
            paper_only: true,
        },
        OwnerAttentionAction {
            action_id: "sprint145-general-comment".to_string(),
            item_id: "owner-general-comment".to_string(),
            action_type: OwnerAttentionActionType::AddComment,
            comment: Some("확인함".to_string()),
            created_at: None,
            paper_only: true,
        },
    ];
    let action_path = root.join("actions.json");
    std::fs::write(
        &action_path,
        serde_json::to_string_pretty(&OwnerActionFile {
            schema_version: "owner-action-file.v1".to_string(),
            source_snapshot_id: Some("sprint145-behavior".to_string()),
            actions,
            paper_only: true,
            safety_notes: vec!["paper-only local triage input".to_string()],
        })
        .expect("action file json"),
    )
    .expect("write action file");
    let result = consume_owner_action_file_with_previous_run(
        OwnerActionConsumptionConfig {
            action_file_path: action_path.display().to_string(),
            processed_action_ledger_path: Some(root.join("ledger.json").display().to_string()),
            inbox_input_path: None,
            inbox_output_path: Some(root.join("inbox.json").display().to_string()),
            watchlist_input_path: None,
            watchlist_output_path: Some(root.join("watchlist.json").display().to_string()),
            member_state_input_path: None,
            member_state_output_path: None,
            committee_state_export_root_path: None,
            owner_console_read_model_path: None,
            apply_mode: OwnerActionApplyMode::Apply,
            duplicate_policy: OwnerActionDuplicatePolicy::SkipAlreadyProcessed,
            after_apply_file_policy: OwnerActionAfterApplyFilePolicy::KeepActionFile,
            paper_only: true,
        },
        Some(run),
    )
    .expect("consume behavior actions");
    assert!(result.action_results.iter().any(|result| {
        result.action_id == "sprint145-ack"
            && result.new_status == OwnerAttentionInboxStatus::Acknowledged
    }));
    assert!(result.action_results.iter().any(|result| {
        result.action_id == "sprint145-defer"
            && result.new_status == OwnerAttentionInboxStatus::Deferred
    }));
    assert!(result.action_results.iter().any(|result| {
        result.action_id == "sprint145-dismiss"
            && result.new_status == OwnerAttentionInboxStatus::Dismissed
    }));
    assert!(result.action_results.iter().any(|result| {
        result.action_id == "sprint145-watch"
            && result.new_status == OwnerAttentionInboxStatus::ConvertedToWatchlist
    }));
    assert!(!result.generated_watchlist_candidates.is_empty());
    assert!(
        result
            .generated_watchlist_candidates
            .iter()
            .all(|candidate| candidate.paper_only)
    );
    assert!(
        result.generated_owner_feedback.iter().any(|feedback| {
            feedback.feedback_type == OwnerFeedbackType::ReconsiderationRequest
        })
    );
    assert!(
        result
            .generated_owner_feedback
            .iter()
            .any(|feedback| feedback.feedback_type == OwnerFeedbackType::Comment)
    );
    assert!(
        result
            .reconsideration_results
            .iter()
            .any(|item| item.chairman_reconsideration_decision_count > 0)
    );
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn owner_action_consumption_rejects_paths_and_unsafe_files() {
    let remote_err = consume_owner_action_file(OwnerActionConsumptionConfig {
        action_file_path: "https://example.invalid/actions.json".to_string(),
        processed_action_ledger_path: None,
        inbox_input_path: None,
        inbox_output_path: None,
        watchlist_input_path: None,
        watchlist_output_path: None,
        member_state_input_path: None,
        member_state_output_path: None,
        committee_state_export_root_path: None,
        owner_console_read_model_path: None,
        apply_mode: OwnerActionApplyMode::DryRun,
        duplicate_policy: OwnerActionDuplicatePolicy::SkipAlreadyProcessed,
        after_apply_file_policy: OwnerActionAfterApplyFilePolicy::KeepActionFile,
        paper_only: true,
    })
    .expect_err("remote action path rejected");
    assert!(remote_err.contains("local"));
    let traversal_err = consume_owner_action_file(OwnerActionConsumptionConfig {
        action_file_path: "target/../actions.json".to_string(),
        processed_action_ledger_path: None,
        inbox_input_path: None,
        inbox_output_path: None,
        watchlist_input_path: None,
        watchlist_output_path: None,
        member_state_input_path: None,
        member_state_output_path: None,
        committee_state_export_root_path: None,
        owner_console_read_model_path: None,
        apply_mode: OwnerActionApplyMode::DryRun,
        duplicate_policy: OwnerActionDuplicatePolicy::SkipAlreadyProcessed,
        after_apply_file_policy: OwnerActionAfterApplyFilePolicy::KeepActionFile,
        paper_only: true,
    })
    .expect_err("traversal action path rejected");
    assert!(traversal_err.contains("parent-directory traversal"));

    let root = std::path::Path::new("target/sprint145_unsafe_action_file");
    let _ = std::fs::remove_dir_all(root);
    std::fs::create_dir_all(root).expect("create unsafe root");
    let unsafe_path = root.join("actions.json");
    std::fs::write(
        &unsafe_path,
        r#"{"schema_version":"owner-action-file.v1","paper_only":true,"order_id":"bad","actions":[]}"#,
    )
    .expect("write unsafe action file");
    let unsafe_err = consume_owner_action_file(OwnerActionConsumptionConfig {
        action_file_path: unsafe_path.display().to_string(),
        processed_action_ledger_path: None,
        inbox_input_path: None,
        inbox_output_path: None,
        watchlist_input_path: None,
        watchlist_output_path: None,
        member_state_input_path: None,
        member_state_output_path: None,
        committee_state_export_root_path: None,
        owner_console_read_model_path: None,
        apply_mode: OwnerActionApplyMode::DryRun,
        duplicate_policy: OwnerActionDuplicatePolicy::SkipAlreadyProcessed,
        after_apply_file_policy: OwnerActionAfterApplyFilePolicy::KeepActionFile,
        paper_only: true,
    })
    .expect_err("unsafe action file rejected");
    assert!(unsafe_err.contains("unsafe field"));
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn member_core_contract_remains_deferred_and_viewer_adds_no_web_dependency() {
    let spec = Mamba3GatedDeltaNetCoreSpec::runtime_deferred_for("contract-lock-member");
    assert_eq!(spec.core_family, MemberCoreFamily::Mamba3GatedDeltaNet);
    assert_eq!(spec.sequence_core, SequenceCoreKind::Mamba3Deferred);
    assert_eq!(spec.memory_core, MemoryCoreKind::GatedDeltaNetDeferred);
    assert_eq!(spec.runtime_status, CoreRuntimeStatus::RuntimeDeferred);
    assert!(
        spec.notes
            .iter()
            .any(|note| note.contains("runtime/training/live inference deferred"))
    );
    let policy = mac_mini_local_policy();
    assert!(policy.do_not_run_all_18_cores_concurrently);
    assert!(policy.lazy_activation);
    assert!(
        policy
            .notes
            .iter()
            .any(|note| note.contains("owner console viewer is read-only UI data"))
    );

    let manifest = std::fs::read_to_string("Cargo.toml").expect("Cargo manifest");
    for forbidden in ["tauri", "svelte", "react", "javascript", "typescript"] {
        assert!(!manifest.to_ascii_lowercase().contains(forbidden));
    }
    let mut forbidden_web_ui_files = Vec::new();
    collect_forbidden_web_ui_files(std::path::Path::new("."), &mut forbidden_web_ui_files)
        .expect("scan web UI files");
    assert!(
        forbidden_web_ui_files.is_empty(),
        "web UI files must not be added: {:?}",
        forbidden_web_ui_files
    );
}

fn collect_forbidden_web_ui_files(
    root: &std::path::Path,
    matches: &mut Vec<std::path::PathBuf>,
) -> Result<(), String> {
    for entry in std::fs::read_dir(root).map_err(|err| err.to_string())? {
        let entry = entry.map_err(|err| err.to_string())?;
        let path = entry.path();
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if path.is_dir() {
            if matches!(
                name.as_ref(),
                ".git" | "target" | "node_modules" | ".svelte-kit" | "dist"
            ) {
                continue;
            }
            collect_forbidden_web_ui_files(&path, matches)?;
            continue;
        }
        let lower_name = name.to_ascii_lowercase();
        let lower_ext = path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase();
        if matches!(
            lower_name.as_str(),
            "package.json"
                | "package-lock.json"
                | "pnpm-lock.yaml"
                | "yarn.lock"
                | "vite.config.js"
                | "vite.config.ts"
                | "svelte.config.js"
                | "tauri.conf.json"
        ) || matches!(lower_ext.as_str(), "js" | "jsx" | "ts" | "tsx" | "svelte")
        {
            matches.push(path);
        }
    }
    Ok(())
}

#[test]
fn daily_brief_storage_update_saves_snapshot_without_running_new_analysis() {
    let run = run_autonomous_paper_committee_loop_from_config_path(std::path::Path::new(
        "examples/soma_minimal_ai_committee_core.toml",
    ))
    .expect("autonomous run for storage update");
    let recheck = run.watchlist_recheck.clone().expect("watchlist recheck");
    let brief = recheck
        .owner_daily_brief
        .clone()
        .expect("owner daily brief");
    let store_path = "target/sprint141_storage_update_briefs.json";
    let snapshot_path = "target/sprint141_storage_update_snapshot.json";
    let _ = std::fs::remove_file(store_path);
    let _ = std::fs::remove_file(snapshot_path);
    let result = run_daily_brief_storage_update(DailyBriefStorageUpdateInput {
        owner_daily_brief: Some(brief.clone()),
        owner_daily_brief_store_input_path: None,
        owner_daily_brief_store_output_path: Some(store_path.to_string()),
        committee_state_snapshot_output_path: Some(snapshot_path.to_string()),
        emit_committee_state_snapshot: true,
        export_input: CommitteeStateExportInput {
            autonomous_run_result: Some(run.clone()),
            watchlist_recheck_result: Some(recheck.clone()),
            attention_inbox: run
                .owner_attention_triage
                .as_ref()
                .map(|triage| triage.inbox.clone()),
            watchlist_store: Some(recheck.updated_watchlist_store.clone()),
            member_state_store: Some(MemberStateStore {
                store_id: "storage-update-member-state-store".to_string(),
                members: run.final_member_states.clone(),
                source_label: "unit-test".to_string(),
                paper_only: true,
            }),
            owner_daily_brief_store: None,
            owner_summary: Some(recheck.owner_summary.clone()),
            owner_console_view: recheck.owner_console_view.clone(),
        },
    })
    .expect("storage update");
    assert_eq!(
        result
            .updated_brief_store
            .latest()
            .map(|item| &item.brief_id),
        Some(&brief.brief_id)
    );
    assert!(result.committee_state_snapshot.is_some());
    assert!(result.safety_summary.no_broker_order_account);
    assert!(std::path::Path::new(store_path).exists());
    assert!(std::path::Path::new(snapshot_path).exists());
    let _ = std::fs::remove_file(store_path);
    let _ = std::fs::remove_file(snapshot_path);
}

#[test]
fn autonomous_paper_config_rejects_remote_and_unsafe_fields() {
    let config = MinimalAiCommitteeCycleConfig {
        input_path: Some("examples/minimal_ai_committee_core_sample.json".to_string()),
        offline_member_opinion_path: None,
        offline_member_output_batch_path: None,
        batch_mode: true,
        member_state_input_path: None,
        member_state_output_path: None,
        emit_owner_summary: true,
        emit_owner_console_view: true,
        owner_feedback_path: None,
        emit_reconsideration_view: false,
        autonomous_paper_run: true,
        run_id: Some("remote-reject".to_string()),
        market_scopes: Vec::new(),
        symbols: Vec::new(),
        max_cycles: 1,
        cycle_mode: AutonomousPaperCycleMode::SingleShot,
        require_owner_confirmation: OwnerConfirmationPolicy::Never,
        local_market_data_path: Some("https://example.invalid/market.json".to_string()),
        local_news_path: None,
        paper_only: true,
        owner_attention_inbox_input_path: None,
        owner_attention_inbox_output_path: None,
        owner_attention_actions_path: None,
        watchlist_candidate_input_path: None,
        watchlist_candidate_output_path: None,
        emit_owner_attention_inbox: false,
        enable_watchlist_recheck: false,
        watchlist_input_path: None,
        watchlist_output_path: None,
        max_candidates_per_cycle: 3,
        include_risk_blocked: false,
        include_needs_evidence: true,
        emit_owner_daily_brief: false,
        owner_daily_brief_store_input_path: None,
        owner_daily_brief_store_output_path: None,
        committee_state_snapshot_output_path: None,
        emit_committee_state_snapshot: false,
        committee_state_export_root_path: None,
        write_latest_snapshot: false,
        write_history_snapshot: false,
        write_snapshot_index: false,
        write_owner_console_read_model: false,
        committee_state_schema_version: None,
        max_snapshot_history_entries: None,
        inline_offline_member_opinions: Vec::new(),
        inline_input: None,
        pilot_roster: None,
        paper_outcome: None,
        archetype_style_cards_path: None,
        style_mapping_mode: StyleMappingMode::None,
    };
    let err = config
        .validate()
        .expect_err("remote autonomous market path must fail");
    assert!(err.contains("local_market_data_path must be local"));

    let unsafe_config_path = std::path::Path::new("target/sprint135_unsafe_autonomous.toml");
    std::fs::write(
        unsafe_config_path,
        r#"
input_path = "examples/minimal_ai_committee_multi_market_sample.json"
batch_mode = true
autonomous_paper_run = true
run_id = "unsafe-autonomous-test"
max_cycles = 1
broker = "not allowed"
"#,
    )
    .expect("write unsafe autonomous config");
    let err = MinimalAiCommitteeCycleConfig::from_toml_path(unsafe_config_path)
        .expect_err("unsafe autonomous config must fail");
    assert!(err.contains("unsafe field or instruction"));
    let _ = std::fs::remove_file(unsafe_config_path);
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

mod common;
#[path = "support/official_committee_support.rs"]
mod official_committee_support;
mod support;

use serde_json::Value;
use soma_zero::{
    CommitteeReferenceClosureStatus, CommitteeReferencePackConfig,
    CommitteeVsPrototypeReferenceClosureStatus, DefensiveCounterfactualDepthStatus,
    EvidenceWeightedDecisionGateV2Status, GeneratedCommitteeReference,
    GeneratedCommitteeReferencePack, GeneratedReferenceKind, GeneratedReferenceSource,
    GeneratedReferenceStatus, OfficialBaselineReferencePackV3Status,
    OfficialEvidenceDepthExpansionBundle, OfficialEvidenceDepthExpansionRunner,
    OfficialEvidenceDepthExpansionStatus, OfficialNoTradeCounterfactualPackV3Status,
    OfficialOutcomeReferencePackV3Status, OfficialReferenceDiversityStatus,
    OfficialReferenceNoLookaheadAuditStatus, OfficialReferenceQualityStatus,
    OfficialReferenceSourceBoundaryAuditStatus, OfficialRiskDeniedCounterfactualPackV3Status,
    OfficialScenarioReferencePackV3Status, ReferencePackQualityStatus,
    SequenceCoreConfidenceRerunStatus, TrainingArtifactReferenceDepthClosureStatus,
    build_reference_pack_quality_report,
};
use support::{shared_fixture_harness as harness, sprint69_support as sprint};

fn pack_with_counts(
    name: &str,
    outcome: usize,
    baseline: usize,
    no_trade: usize,
    risk_denied: usize,
    diagnostic: usize,
    fixture: bool,
) -> GeneratedCommitteeReferencePack {
    let mut row = official_committee_support::scenario_row(name, 0, "AAPL", 1_700_000_000_000);
    if fixture {
        row.source_kind = soma_zero::CommitteeScenarioSourceKind::Fixture;
        row.evidence_source_kind = soma_zero::EvidenceSourceKind::TestFixture;
    }
    let scenario_rows = vec![row.clone()];
    let mut references = Vec::new();
    for index in 0..outcome {
        references.push(GeneratedCommitteeReference {
            reference_id: format!("o-{index}"),
            scenario_row_id: row.scenario_row_id.clone(),
            reference_kind: GeneratedReferenceKind::TripleBarrierOutcome,
            status: GeneratedReferenceStatus::Generated,
            outcome_reference: Some(soma_zero::CommitteeOutcomeReference {
                outcome_id: format!("o-{index}"),
                decision_id: None,
                symbol: row.symbol.clone(),
                timestamp_ms: row.timestamp_ms,
                horizon_bars: 24,
                triple_barrier_label: soma_zero::CommitteeTripleBarrierLabel::TakeProfit,
                net_return_pct: Some(0.01),
                max_favorable_excursion_pct: Some(0.01),
                max_adverse_excursion_pct: Some(0.0),
                cost_bps: 5.0,
                slippage_bps: 2.0,
                source_kind: row.evidence_source_kind,
                no_lookahead_safe: true,
                reason_codes: vec![],
            }),
            baseline_reference: None,
            external_reference: None,
            no_trade_counterfactual: None,
            risk_denied_counterfactual: None,
            generated_from: GeneratedReferenceSource::LocalCandleSeries,
            official_readiness_eligible: !fixture,
            diagnostic_only: diagnostic > 0,
            reason_codes: vec![],
        });
    }
    for index in 0..baseline {
        references.push(GeneratedCommitteeReference {
            reference_id: format!("b-{index}"),
            scenario_row_id: row.scenario_row_id.clone(),
            reference_kind: GeneratedReferenceKind::BaselineAction,
            status: GeneratedReferenceStatus::Generated,
            outcome_reference: None,
            baseline_reference: Some(soma_zero::CommitteeBaselineReference {
                baseline_action: soma_zero::CommitteeBaselineAction::Approve,
                baseline_confidence: Some(0.7),
                baseline_expected_edge: Some(0.01),
                baseline_reason_codes: vec![],
                reason_codes: vec![],
            }),
            external_reference: None,
            no_trade_counterfactual: None,
            risk_denied_counterfactual: None,
            generated_from: GeneratedReferenceSource::DeterministicBaselinePolicy,
            official_readiness_eligible: false,
            diagnostic_only: diagnostic > 0,
            reason_codes: vec![],
        });
    }
    for index in 0..no_trade {
        references.push(GeneratedCommitteeReference {
            reference_id: format!("n-{index}"),
            scenario_row_id: row.scenario_row_id.clone(),
            reference_kind: GeneratedReferenceKind::NoTradeCounterfactual,
            status: GeneratedReferenceStatus::Generated,
            outcome_reference: None,
            baseline_reference: None,
            external_reference: None,
            no_trade_counterfactual: Some(soma_zero::CommitteeCounterfactualRecord {
                counterfactual_id: format!("n-{index}"),
                scenario_row_id: row.scenario_row_id.clone(),
                counterfactual_type: soma_zero::CommitteeCounterfactualType::NoTrade,
                build_status: soma_zero::CounterfactualBuildStatus::Built,
                triple_barrier_label: Some(
                    soma_zero::CommitteeTripleBarrierLabel::NoTradeCounterfactual,
                ),
                net_return_pct: Some(0.0),
                avoided_loss_value: None,
                missed_gain_value: None,
                max_favorable_excursion_pct: Some(0.0),
                max_adverse_excursion_pct: Some(0.0),
                cost_bps: 5.0,
                slippage_bps: 2.0,
                no_lookahead_safe: true,
                diagnostic_only: diagnostic > 0,
                reason_codes: vec![],
            }),
            risk_denied_counterfactual: None,
            generated_from: GeneratedReferenceSource::LocalCandleSeries,
            official_readiness_eligible: !fixture,
            diagnostic_only: diagnostic > 0,
            reason_codes: vec![],
        });
    }
    for index in 0..risk_denied {
        references.push(GeneratedCommitteeReference {
            reference_id: format!("r-{index}"),
            scenario_row_id: row.scenario_row_id.clone(),
            reference_kind: GeneratedReferenceKind::RiskDeniedCounterfactual,
            status: GeneratedReferenceStatus::Generated,
            outcome_reference: None,
            baseline_reference: None,
            external_reference: None,
            no_trade_counterfactual: None,
            risk_denied_counterfactual: Some(soma_zero::CommitteeCounterfactualRecord {
                counterfactual_id: format!("r-{index}"),
                scenario_row_id: row.scenario_row_id.clone(),
                counterfactual_type: soma_zero::CommitteeCounterfactualType::RiskDenied,
                build_status: soma_zero::CounterfactualBuildStatus::Built,
                triple_barrier_label: Some(
                    soma_zero::CommitteeTripleBarrierLabel::RiskDeniedCounterfactual,
                ),
                net_return_pct: Some(0.0),
                avoided_loss_value: None,
                missed_gain_value: None,
                max_favorable_excursion_pct: Some(0.0),
                max_adverse_excursion_pct: Some(0.0),
                cost_bps: 5.0,
                slippage_bps: 2.0,
                no_lookahead_safe: true,
                diagnostic_only: diagnostic > 0,
                reason_codes: vec![],
            }),
            generated_from: GeneratedReferenceSource::LocalCandleSeries,
            official_readiness_eligible: !fixture,
            diagnostic_only: diagnostic > 0,
            reason_codes: vec![],
        });
    }
    GeneratedCommitteeReferencePack::new(
        name.to_string(),
        scenario_rows,
        references,
        soma_zero::CandleAlignmentReport {
            records: vec![],
            matched_count: 1,
            unmatched_count: 0,
            exact_match_count: 1,
            tolerance_match_count: 0,
            missing_series_count: 0,
            missing_timestamp_count: 0,
            wrong_symbol_count: 0,
            insufficient_future_bars_count: 0,
            no_lookahead_rejected_count: 0,
            alignment_status: soma_zero::CandleAlignmentOverallStatus::HealthyAlignment,
            reason_codes: vec![],
        },
        vec![],
    )
}

#[test]
fn sprint82_bundle_conservative_coverage_preserved() {
    let bundle = sprint::run_sprint82_bundle(
        "soma_official_evidence_depth_expand.toml",
        "sprint82-evidence-depth-suite",
    );
    let expected: Value = harness::load_json_fixture(sprint::example_path(
        "sprint82_data/expected_evidence_depth_summary.json",
    ));

    assert_eq!(
        bundle
            .official_evidence_depth_expansion_report
            .expansion_status,
        OfficialEvidenceDepthExpansionStatus::OfficialEvidenceDepthExpanded
    );
    assert_eq!(
        bundle.committee_reference_closure_report.closure_status,
        CommitteeReferenceClosureStatus::CommitteeReferencesImproved
    );
    assert_eq!(
        bundle.official_scenario_reference_pack_v3.pack_status,
        OfficialScenarioReferencePackV3Status::ScenarioReferencePackReady
    );
    assert_eq!(
        bundle.official_outcome_reference_pack_v3.pack_status,
        OfficialOutcomeReferencePackV3Status::OutcomeReferencePackReady
    );
    assert_eq!(
        bundle.official_baseline_reference_pack_v3.pack_status,
        OfficialBaselineReferencePackV3Status::BaselineReferencePackReady
    );
    assert_eq!(
        bundle.official_no_trade_counterfactual_pack_v3.pack_status,
        OfficialNoTradeCounterfactualPackV3Status::NoTradeCounterfactualPackReady
    );
    assert_eq!(
        bundle
            .official_risk_denied_counterfactual_pack_v3
            .pack_status,
        OfficialRiskDeniedCounterfactualPackV3Status::RiskDeniedCounterfactualPackReady
    );
    assert!(matches!(
        bundle.defensive_counterfactual_depth_report.depth_status,
        DefensiveCounterfactualDepthStatus::DefensiveDepthReady
            | DefensiveCounterfactualDepthStatus::DefensiveDepthReadyWithWarnings
    ));
    assert_eq!(
        bundle.official_reference_quality_report.quality_status,
        OfficialReferenceQualityStatus::ReferenceQualityReadyWithWarnings
    );
    assert_eq!(
        bundle.official_reference_diversity_report.diversity_status,
        OfficialReferenceDiversityStatus::ReferenceDiversityReady
    );
    assert_eq!(
        bundle.official_reference_no_lookahead_audit.audit_status,
        OfficialReferenceNoLookaheadAuditStatus::NoLookaheadReady
    );
    assert_eq!(
        bundle.official_reference_source_boundary_audit.audit_status,
        OfficialReferenceSourceBoundaryAuditStatus::SourceBoundaryReady
    );
    assert_eq!(
        bundle.sequence_core_confidence_rerun_report.rerun_status,
        SequenceCoreConfidenceRerunStatus::ConfidenceImproved
    );
    assert_eq!(
        bundle.evidence_weighted_decision_gate_v2.decision_status,
        EvidenceWeightedDecisionGateV2Status::KeepBothAsResearchCandidates
    );
    assert_eq!(
        bundle
            .committee_vs_prototype_reference_closure_report
            .closure_status,
        CommitteeVsPrototypeReferenceClosureStatus::ComparisonReferencesImproved
    );
    assert_eq!(
        bundle
            .training_artifact_reference_depth_closure_report
            .closure_status,
        TrainingArtifactReferenceDepthClosureStatus::TrainingReferenceDepthImproved
    );
    assert!(!bundle.evidence_weighted_decision_gate_v2.runtime_allowed);
    assert!(!bundle.evidence_weighted_decision_gate_v2.training_allowed);
    assert!(
        !bundle
            .evidence_weighted_decision_gate_v2
            .live_inference_allowed
    );
    assert_eq!(
        expected["official_evidence_depth"].as_str(),
        Some("OfficialEvidenceDepthExpanded")
    );
    harness::assert_no_secret_like_values(&bundle.final_summary);
    harness::assert_no_order_account_fields(&bundle.final_summary);
    harness::assert_no_runtime_fields(&bundle.final_summary);
}

#[test]
fn sprint82_config_defaults_and_remote_guard_hold() {
    let config = sprint::sprint82_evidence_config_from_example(
        "soma_official_evidence_depth_expand.toml",
        "sprint82-evidence-depth-config",
    );
    assert!(config.prefer_local_official_evidence);
    assert!(config.require_provenance);
    assert!(config.require_preflight);
    assert!(config.require_no_lookahead);
    assert!(config.require_source_class);

    let mut remote = config.clone();
    remote.real_evidence_paths = vec!["https://example.com/evidence.json".to_string()];
    let error = remote.validate().expect_err("remote paths rejected");
    assert!(error.contains("must be local"));
}

#[test]
fn sprint82_weak_official_depth_stays_conservative() {
    let weak_snapshot: Value = harness::load_json_fixture(sprint::example_path(
        "sprint82_data/official_evidence_depth_before.json",
    ));
    let before = sprint::write_support_json("sprint82-weak-before", "before.json", &weak_snapshot);
    let after = sprint::write_support_json("sprint82-weak-after", "after.json", &weak_snapshot);
    let confidence_before = sprint::write_support_json(
        "sprint82-weak-confidence-before",
        "confidence_before.json",
        &serde_json::json!({
            "confidence_status": "InsufficientEvidence",
            "mamba3fin_confidence": "Insufficient",
            "gated_deltanet_confidence": "Insufficient",
            "committee_confidence": "Low",
            "no_trade_confidence": "Low",
            "risk_denied_confidence": "Low"
        }),
    );
    let confidence_after = sprint::write_support_json(
        "sprint82-weak-confidence-after",
        "confidence_after.json",
        &serde_json::json!({
            "confidence_status": "InsufficientEvidence",
            "mamba3fin_confidence": "Insufficient",
            "gated_deltanet_confidence": "Insufficient",
            "committee_confidence": "Low",
            "no_trade_confidence": "Low",
            "risk_denied_confidence": "Low"
        }),
    );

    let mut config = sprint::sprint82_evidence_config_from_example(
        "soma_official_evidence_depth_expand.toml",
        "sprint82-evidence-depth-weak",
    );
    config.real_evidence_paths = vec![before, after, confidence_before, confidence_after];

    let bundle = OfficialEvidenceDepthExpansionRunner::default()
        .run(&config)
        .expect("weak evidence bundle");
    assert_eq!(
        bundle
            .official_evidence_depth_expansion_report
            .expansion_status,
        OfficialEvidenceDepthExpansionStatus::NeedMoreOfficialEvidence
    );
    assert_eq!(
        bundle.evidence_weighted_decision_gate_v2.decision_status,
        EvidenceWeightedDecisionGateV2Status::NeedMoreOfficialEvidence
    );
}

#[test]
fn sprint82_reference_pack_quality_is_preserved_and_deterministic() {
    let config = CommitteeReferencePackConfig::default();
    assert_eq!(
        build_reference_pack_quality_report(&config, &pack_with_counts("q1", 0, 1, 1, 1, 0, false))
            .quality_status,
        ReferencePackQualityStatus::NeedMoreOutcomeReferences
    );
    assert_eq!(
        build_reference_pack_quality_report(&config, &pack_with_counts("q2", 1, 0, 1, 1, 0, false))
            .quality_status,
        ReferencePackQualityStatus::NeedMoreBaselineReferences
    );
    assert_eq!(
        build_reference_pack_quality_report(&config, &pack_with_counts("q3", 1, 1, 0, 1, 0, false))
            .quality_status,
        ReferencePackQualityStatus::NeedMoreNoTradeCounterfactuals
    );
    assert_eq!(
        build_reference_pack_quality_report(&config, &pack_with_counts("q4", 1, 1, 1, 0, 0, false))
            .quality_status,
        ReferencePackQualityStatus::NeedMoreRiskDeniedCounterfactuals
    );

    let mut diagnostic_pack = pack_with_counts("q5", 1, 1, 1, 1, 4, false);
    diagnostic_pack.diagnostic_only_count = 4;
    assert_eq!(
        build_reference_pack_quality_report(&config, &diagnostic_pack).quality_status,
        ReferencePackQualityStatus::TooManyDiagnosticOnlyReferences
    );

    let first = build_reference_pack_quality_report(
        &CommitteeReferencePackConfig {
            allow_controlled_fixture_references: true,
            ..CommitteeReferencePackConfig::default()
        },
        &pack_with_counts("q6", 1, 1, 1, 1, 0, true),
    );
    let second = build_reference_pack_quality_report(
        &CommitteeReferencePackConfig {
            allow_controlled_fixture_references: true,
            ..CommitteeReferencePackConfig::default()
        },
        &pack_with_counts("q6", 1, 1, 1, 1, 0, true),
    );
    assert_eq!(first, second);
    assert_eq!(
        first.quality_status,
        ReferencePackQualityStatus::HealthyReferencePack
    );
}

#[test]
fn sprint82_cli_help_and_local_only_guards_are_present() {
    let bin = env!("CARGO_BIN_EXE_soma_experiment");
    let expected = [
        (
            "official-evidence-depth-expand",
            "offline-only official evidence depth expansion",
        ),
        (
            "committee-reference-close",
            "Trinity-only committee reference closure",
        ),
        (
            "official-scenario-pack-v3",
            "scenario-only official pack v3",
        ),
        ("official-outcome-pack-v3", "outcome-only official pack v3"),
        (
            "official-baseline-pack-v3",
            "baseline-only official pack v3",
        ),
        (
            "official-notrade-pack-v3",
            "NoTrade-only official counterfactual pack v3",
        ),
        (
            "official-riskdenied-pack-v3",
            "RiskDenied-only official counterfactual pack v3",
        ),
        (
            "defensive-counterfactual-depth",
            "defensive counterfactual depth only",
        ),
        (
            "official-reference-quality",
            "official reference quality only",
        ),
        (
            "official-reference-diversity",
            "official reference diversity only",
        ),
        (
            "official-reference-no-lookahead",
            "official no-lookahead audit only",
        ),
        (
            "official-reference-source-boundary",
            "official source-boundary audit only",
        ),
        (
            "sequence-core-confidence-rerun",
            "sequence-core confidence rerun only",
        ),
        (
            "sequence-core-decision-gate-v2",
            "evidence-weighted decision gate v2",
        ),
        (
            "control-tower-evidence-depth",
            "read-only official evidence depth panel",
        ),
    ];
    for (command, text) in expected {
        let help = std::process::Command::new(bin)
            .args([command, "--help"])
            .output()
            .expect("help");
        assert!(help.status.success());
        let stdout = String::from_utf8(help.stdout).expect("stdout");
        assert!(stdout.contains("--config"));
        assert!(stdout.to_lowercase().contains(&text.to_lowercase()));
    }

    let root_help = std::process::Command::new(bin)
        .arg("--help")
        .output()
        .expect("root help");
    let root_stdout = String::from_utf8(root_help.stdout).expect("stdout");
    assert!(root_stdout.contains("official-evidence-depth-expand"));
    assert!(root_stdout.contains("committee-reference-close"));
    assert!(root_stdout.contains("sequence-core-decision-gate-v2"));
    assert!(root_stdout.contains("control-tower-evidence-depth"));
    assert!(!root_stdout.contains("live-inference"));
    assert!(!root_stdout.contains("train-model"));

    for command in [
        "official-evidence-depth-expand",
        "committee-reference-close",
        "official-scenario-pack-v3",
        "official-outcome-pack-v3",
        "official-baseline-pack-v3",
        "official-notrade-pack-v3",
        "official-riskdenied-pack-v3",
        "defensive-counterfactual-depth",
        "official-reference-quality",
        "official-reference-diversity",
        "official-reference-no-lookahead",
        "official-reference-source-boundary",
        "sequence-core-confidence-rerun",
        "sequence-core-decision-gate-v2",
        "control-tower-evidence-depth",
    ] {
        let remote = std::process::Command::new(bin)
            .args([command, "--config", "https://example.com/sprint82.toml"])
            .output()
            .expect("remote config");
        assert!(!remote.status.success());
        let stderr = String::from_utf8(remote.stderr).expect("stderr");
        assert!(stderr.contains("must be local"));
    }
}

#[test]
fn sprint82_grouped_suite_is_deterministic() {
    let first: OfficialEvidenceDepthExpansionBundle = sprint::run_sprint82_bundle(
        "soma_official_evidence_depth_expand.toml",
        "sprint82-evidence-depth-determinism-a",
    );
    let second: OfficialEvidenceDepthExpansionBundle = sprint::run_sprint82_bundle(
        "soma_official_evidence_depth_expand.toml",
        "sprint82-evidence-depth-determinism-b",
    );
    assert_eq!(first, second);
}

mod common;

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use serde_json::json;
use soma_zero::{
    ArtifactSize, CommitteeValueAttributionInputs, CommitteeValueAttributionStatus,
    CoreBottleneckInputs, CoreBottleneckKind, CoreBottleneckRecommendation,
    CoreLatencyBudgetConfig, CoreLatencyBudgetStatus, CorePerformanceArtifactInventory,
    CorePerformanceFinalStatus, CorePerformanceRegressionConfig, CorePerformanceRegressionReport,
    CorePerformanceRegressionSummary, CorePerformanceScorecardConfig, NoTradeValueInputs,
    NoTradeValueStatus, RiskGovernorValueInputs, RiskGovernorValueStatus, SignalQualityInputs,
    SignalQualityStatus, build_committee_value_attribution_report, build_core_bottleneck_report,
    build_core_latency_budget_report, build_core_performance_regression_report,
    build_no_trade_value_report, build_risk_governor_value_report, build_signal_quality_report,
};

fn write_json(name: &str, file_name: &str, value: serde_json::Value) -> PathBuf {
    let path = common::output_dir(name).join(file_name);
    fs::write(&path, serde_json::to_string_pretty(&value).expect("json")).expect("write json");
    path
}

fn write_text(name: &str, file_name: &str, contents: &str) -> PathBuf {
    let path = common::output_dir(name).join(file_name);
    fs::write(&path, contents).expect("write text");
    path
}

#[test]
fn core_performance_config_validates_defaults_limits_and_examples() {
    let config = CorePerformanceScorecardConfig::default();
    assert!(config.require_core_check_pass);
    assert!(config.require_official_for_usefulness_claim);
    assert!(config.allow_controlled_evidence);
    assert!(config.allow_crypto_only);
    assert!(config.allow_yfinance_research);
    assert!(config.allow_fixture);
    config.validate().expect("default config");

    let remote = CorePerformanceScorecardConfig {
        scorecard_id: "remote".to_string(),
        core_check_report_paths: vec!["https://example.com/core.json".to_string()],
        ..CorePerformanceScorecardConfig::default()
    };
    assert!(remote.validate().unwrap_err().contains("local"));

    let too_many_artifacts = CorePerformanceScorecardConfig {
        scorecard_id: "artifacts".to_string(),
        max_artifacts: 0,
        ..CorePerformanceScorecardConfig::default()
    };
    assert!(
        too_many_artifacts
            .validate()
            .unwrap_err()
            .contains("max_artifacts")
    );

    let too_many_rows = CorePerformanceScorecardConfig {
        scorecard_id: "rows".to_string(),
        max_rows: 100_001,
        ..CorePerformanceScorecardConfig::default()
    };
    assert!(too_many_rows.validate().unwrap_err().contains("max_rows"));

    let too_many_bytes = CorePerformanceScorecardConfig {
        scorecard_id: "bytes".to_string(),
        max_bytes: 20_000_001,
        ..CorePerformanceScorecardConfig::default()
    };
    assert!(too_many_bytes.validate().unwrap_err().contains("max_bytes"));

    assert!(
        CorePerformanceScorecardConfig::from_toml_str(
            "scorecard_id = 'bad'\nbroker = 'forbidden'\n"
        )
        .is_err()
    );

    let examples = [
        "examples/soma_core_performance_controlled.toml",
        "examples/soma_core_performance_crypto_only.toml",
        "examples/soma_core_performance_official_replication.toml",
        "examples/soma_core_performance_diagnostics_only.toml",
    ];
    for example in examples {
        CorePerformanceScorecardConfig::from_toml_path(&PathBuf::from(example))
            .expect("parse scorecard example");
    }

    CorePerformanceRegressionConfig::from_toml_path(&PathBuf::from(
        "examples/soma_core_regression.toml",
    ))
    .expect("parse regression example");
    CorePerformanceScorecardConfig::from_toml_path(&PathBuf::from(
        "examples/soma_core_bottleneck.toml",
    ))
    .expect("parse bottleneck example");
}

#[test]
fn artifact_inventory_detects_known_kinds_and_markers_without_panicking() {
    let official_replication = write_json(
        "core-performance-inventory-official",
        "official_replication_report.json",
        json!({
            "row_injection_result": {"injected_rows": [1, 2, 3]},
            "official_sufficiency_replication_report": {
                "non_crypto_official_row_count": 3,
                "outcome_link_count": 2,
                "baseline_reference_count": 1,
                "no_trade_counterfactual_count": 1
            },
            "symbol": "AAPL",
            "timeframe": "OneDay"
        }),
    );
    let official_benchmark = write_json(
        "core-performance-inventory-benchmark",
        "committee_official_benchmark_report.json",
        json!({
            "outcome_linked_vs_baseline_report": {"outcome_linked_rows": 2},
            "symbol": "AAPL"
        }),
    );
    let outcome_coverage = write_json(
        "core-performance-inventory-controlled",
        "committee_outcome_coverage_controlled.json",
        json!({
            "coverage_report": {"outcome_linked_rows": 2, "baseline_linked_rows": 2},
            "performance_matrix": {"cells": []}
        }),
    );
    let reference_pack = write_json(
        "core-performance-inventory-fixture",
        "committee_reference_pack_fixture.json",
        json!({
            "reference_pack": {"generated_baseline_count": 2}
        }),
    );
    let source_benchmark = write_json(
        "core-performance-inventory-source",
        "source_benchmark_report.json",
        json!({"dataset_inventory": {"official": 1}}),
    );
    let yahoo_report = write_json(
        "core-performance-inventory-yahoo",
        "yahoo_research_report.json",
        json!({"yfinance_symbols": ["AAPL"]}),
    );
    let crypto_only = write_json(
        "core-performance-inventory-crypto",
        "official_replication_crypto_only.json",
        json!({
            "row_injection_result": {"injected_rows": [1]},
            "market": "Crypto",
            "status": "CryptoOnly"
        }),
    );
    let provider_readiness = write_json(
        "core-performance-inventory-readiness",
        "provider_readiness_report.json",
        json!({"catalog": [], "selection_results": []}),
    );
    let unknown = write_json(
        "core-performance-inventory-unknown",
        "mystery_artifact.json",
        json!({"mystery": true}),
    );
    let invalid = write_text(
        "core-performance-inventory-invalid",
        "broken_artifact.json",
        "{ not valid json",
    );

    let paths = vec![
        official_replication.display().to_string(),
        official_benchmark.display().to_string(),
        outcome_coverage.display().to_string(),
        reference_pack.display().to_string(),
        source_benchmark.display().to_string(),
        yahoo_report.display().to_string(),
        crypto_only.display().to_string(),
        provider_readiness.display().to_string(),
        unknown.display().to_string(),
        invalid.display().to_string(),
    ];

    let inventory = CorePerformanceArtifactInventory::from_paths(&paths);
    let second = CorePerformanceArtifactInventory::from_paths(&paths);
    assert_eq!(inventory.to_text(), second.to_text());

    let descriptors = inventory
        .descriptors
        .iter()
        .map(|descriptor| {
            (
                PathBuf::from(&descriptor.path)
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .to_string(),
                descriptor,
            )
        })
        .collect::<BTreeMap<_, _>>();

    let official = descriptors
        .get("official_replication_report.json")
        .expect("official");
    assert!(official.non_crypto_official);
    let benchmark = descriptors
        .get("committee_official_benchmark_report.json")
        .expect("benchmark");
    assert!(benchmark.non_crypto_official);
    let controlled = descriptors
        .get("committee_outcome_coverage_controlled.json")
        .expect("controlled");
    assert!(controlled.controlled_only);
    assert!(!controlled.official);
    let fixture = descriptors
        .get("committee_reference_pack_fixture.json")
        .expect("fixture");
    assert!(fixture.fixture_only);
    assert!(!fixture.official);
    let yahoo = descriptors
        .get("yahoo_research_report.json")
        .expect("yahoo");
    assert!(yahoo.research_only);
    assert!(!yahoo.official);
    let crypto = descriptors
        .get("official_replication_crypto_only.json")
        .expect("crypto");
    assert!(crypto.crypto_only);
    assert!(!crypto.non_crypto_official);
    let provider = descriptors
        .get("provider_readiness_report.json")
        .expect("provider");
    assert!(!provider.official);
    let unknown_descriptor = descriptors.get("mystery_artifact.json").expect("unknown");
    assert!(
        unknown_descriptor
            .reason_codes
            .contains(&soma_zero::ReasonCode::CommitteeArtifactUnknown)
    );

    assert_eq!(inventory.non_crypto_official_count, 2);
    assert_eq!(inventory.research_only_count, 1);
    assert_eq!(inventory.fixture_only_count, 1);
    assert_eq!(inventory.controlled_only_count, 1);
    assert_eq!(inventory.crypto_only_count, 1);
    assert_eq!(inventory.unknown_count, 2);
}

#[test]
fn signal_quality_reports_cover_research_fixture_crypto_baseline_and_calibration_paths() {
    let missing_links = build_signal_quality_report(&SignalQualityInputs {
        evaluated_rows: 4,
        official_evaluated_rows: 0,
        outcome_linked_rows: 0,
        require_official_for_usefulness_claim: true,
        ..SignalQualityInputs::default()
    });
    assert_eq!(
        missing_links.signal_quality_status,
        SignalQualityStatus::EvidenceInsufficient
    );

    let baseline_only = build_signal_quality_report(&SignalQualityInputs {
        evaluated_rows: 4,
        official_evaluated_rows: 4,
        outcome_linked_rows: 4,
        baseline_reference_rows: 4,
        committee_decision_rows: 0,
        ..SignalQualityInputs::default()
    });
    assert_eq!(
        baseline_only.signal_quality_status,
        SignalQualityStatus::BaselineOnlyEvidence
    );

    let committee_available = build_signal_quality_report(&SignalQualityInputs {
        evaluated_rows: 6,
        official_evaluated_rows: 6,
        outcome_linked_rows: 6,
        baseline_reference_rows: 6,
        committee_decision_rows: 6,
        external_reference_rows: 0,
        ..SignalQualityInputs::default()
    });
    assert_eq!(
        committee_available.signal_quality_status,
        SignalQualityStatus::CommitteeEvidenceAvailable
    );

    let poor_calibration = build_signal_quality_report(&SignalQualityInputs {
        evaluated_rows: 6,
        official_evaluated_rows: 6,
        outcome_linked_rows: 6,
        baseline_reference_rows: 6,
        committee_decision_rows: 6,
        brier_score: Some(0.30),
        ..SignalQualityInputs::default()
    });
    assert_eq!(
        poor_calibration.signal_quality_status,
        SignalQualityStatus::PoorCalibration
    );

    assert_eq!(
        build_signal_quality_report(&SignalQualityInputs {
            research_only: true,
            ..SignalQualityInputs::default()
        })
        .signal_quality_status,
        SignalQualityStatus::ResearchOnly
    );
    assert_eq!(
        build_signal_quality_report(&SignalQualityInputs {
            fixture_only: true,
            ..SignalQualityInputs::default()
        })
        .signal_quality_status,
        SignalQualityStatus::FixtureOnly
    );
    assert_eq!(
        build_signal_quality_report(&SignalQualityInputs {
            crypto_only: true,
            ..SignalQualityInputs::default()
        })
        .signal_quality_status,
        SignalQualityStatus::CryptoOnly
    );

    assert_eq!(committee_available.to_text(), committee_available.to_text());
}

#[test]
fn committee_value_reports_cover_diagnostic_and_proxy_statuses() {
    let insufficient =
        build_committee_value_attribution_report(&CommitteeValueAttributionInputs::default());
    assert_eq!(
        insufficient.attribution_status,
        CommitteeValueAttributionStatus::InsufficientComparableRows
    );

    let diagnostic = build_committee_value_attribution_report(&CommitteeValueAttributionInputs {
        comparable_rows: 8,
        official_comparable_rows: 0,
        committee_vs_baseline_delta: Some(0.10),
        diagnostic_only: true,
        ..CommitteeValueAttributionInputs::default()
    });
    assert_eq!(
        diagnostic.attribution_status,
        CommitteeValueAttributionStatus::DiagnosticOnly
    );

    let worse = build_committee_value_attribution_report(&CommitteeValueAttributionInputs {
        comparable_rows: 8,
        official_comparable_rows: 8,
        committee_vs_baseline_delta: Some(-0.05),
        ..CommitteeValueAttributionInputs::default()
    });
    assert_eq!(
        worse.attribution_status,
        CommitteeValueAttributionStatus::CommitteeWorseThanBaseline
    );

    let mostly_no_trade =
        build_committee_value_attribution_report(&CommitteeValueAttributionInputs {
            comparable_rows: 8,
            official_comparable_rows: 8,
            committee_action_counts: BTreeMap::from([("FinalNoTrade".to_string(), 5)]),
            ..CommitteeValueAttributionInputs::default()
        });
    assert_eq!(
        mostly_no_trade.attribution_status,
        CommitteeValueAttributionStatus::CommitteeMostlyNoTrade
    );

    let mostly_denied =
        build_committee_value_attribution_report(&CommitteeValueAttributionInputs {
            comparable_rows: 8,
            official_comparable_rows: 8,
            committee_action_counts: BTreeMap::from([("FinalDenied".to_string(), 5)]),
            ..CommitteeValueAttributionInputs::default()
        });
    assert_eq!(
        mostly_denied.attribution_status,
        CommitteeValueAttributionStatus::CommitteeMostlyRiskDenied
    );

    let chair = build_committee_value_attribution_report(&CommitteeValueAttributionInputs {
        comparable_rows: 8,
        official_comparable_rows: 8,
        chair_dominated: true,
        ..CommitteeValueAttributionInputs::default()
    });
    assert_eq!(
        chair.attribution_status,
        CommitteeValueAttributionStatus::ChairDominated
    );

    let risk = build_committee_value_attribution_report(&CommitteeValueAttributionInputs {
        comparable_rows: 8,
        official_comparable_rows: 8,
        risk_dominated: true,
        ..CommitteeValueAttributionInputs::default()
    });
    assert_eq!(
        risk.attribution_status,
        CommitteeValueAttributionStatus::RiskDominated
    );

    assert_eq!(chair.to_text(), chair.to_text());
}

#[test]
fn risk_governor_and_no_trade_value_reports_stay_conservative() {
    let evidence_weak = build_risk_governor_value_report(&RiskGovernorValueInputs {
        total_decisions: 6,
        approved_count: 1,
        denied_count: 4,
        cooldown_count: 1,
        hard_veto_count: 1,
        evidence_weak: true,
        ..RiskGovernorValueInputs::default()
    });
    assert_eq!(
        evidence_weak.status,
        RiskGovernorValueStatus::RiskDominantBecauseEvidenceWeak
    );
    assert_eq!(evidence_weak.hard_veto_count, 1);

    let overblocking = build_risk_governor_value_report(&RiskGovernorValueInputs {
        total_decisions: 6,
        approved_count: 2,
        denied_count: 3,
        risk_denied_counterfactual_count: 3,
        soft_threshold_denial_count: 2,
        avoided_loss_total: 0.10,
        missed_gain_total: 0.30,
        hard_veto_count: 1,
        ..RiskGovernorValueInputs::default()
    });
    assert_eq!(
        overblocking.status,
        RiskGovernorValueStatus::RiskOverBlockingSuspected
    );
    assert!(overblocking.overblocking_suspected);
    assert_eq!(overblocking.hard_veto_count, 1);

    let no_trade_positive = build_no_trade_value_report(&NoTradeValueInputs {
        no_trade_decisions: 3,
        no_trade_counterfactuals: 3,
        avoided_loss_value: 0.12,
        missed_gain_value: 0.03,
        ..NoTradeValueInputs::default()
    });
    assert_eq!(
        no_trade_positive.status,
        NoTradeValueStatus::NoTradeValuePositive
    );
    assert!((no_trade_positive.no_trade_value_proxy - 0.09).abs() < 1e-9);

    let too_conservative = build_no_trade_value_report(&NoTradeValueInputs {
        no_trade_decisions: 2,
        no_trade_counterfactuals: 2,
        avoided_loss_value: 0.01,
        missed_gain_value: 0.08,
        ..NoTradeValueInputs::default()
    });
    assert_eq!(
        too_conservative.status,
        NoTradeValueStatus::NoTradeTooConservative
    );

    let insufficient = build_no_trade_value_report(&NoTradeValueInputs {
        no_trade_decisions: 2,
        no_trade_counterfactuals: 0,
        ..NoTradeValueInputs::default()
    });
    assert_eq!(
        insufficient.status,
        NoTradeValueStatus::NoTradeInsufficientCounterfactuals
    );
}

#[test]
fn latency_regression_and_bottleneck_reports_are_deterministic_and_reasonable() {
    let latency = build_core_latency_budget_report(
        &CoreLatencyBudgetConfig {
            max_scorecard_artifacts: 2,
            max_rows: 5,
            max_report_bytes: 12,
            max_artifact_bytes: 12,
            max_decision_path_steps: 2,
            target_decision_latency_ms: Some(5),
            ..CoreLatencyBudgetConfig::default()
        },
        &[
            ArtifactSize {
                path: "b.json".to_string(),
                bytes: 3,
            },
            ArtifactSize {
                path: "a.json".to_string(),
                bytes: 9,
            },
            ArtifactSize {
                path: "c.json".to_string(),
                bytes: 1,
            },
        ],
        6,
        13,
        3,
    );
    assert_eq!(latency.artifact_count, 3);
    assert_eq!(latency.row_count, 6);
    assert_eq!(latency.largest_artifacts[0].path, "a.json");
    assert_eq!(
        latency.budget_status,
        CoreLatencyBudgetStatus::TooManyArtifacts
    );

    let previous = CorePerformanceRegressionSummary {
        scorecard_id: "previous".to_string(),
        official_row_count: 10,
        outcome_linked_rows: 10,
        counterfactual_rows: 8,
        brier_score: Some(0.10),
        ece: Some(0.02),
        denial_rate: 0.10,
        avoided_loss_total: 0.30,
        actionability_ratio: Some(0.80),
        report_bytes: 100,
        fingerprint: "fingerprint-a".to_string(),
    };
    let current = CorePerformanceRegressionSummary {
        scorecard_id: "current".to_string(),
        official_row_count: 8,
        outcome_linked_rows: 7,
        counterfactual_rows: 6,
        brier_score: Some(0.14),
        ece: Some(0.05),
        denial_rate: 0.25,
        avoided_loss_total: 0.20,
        actionability_ratio: Some(0.50),
        report_bytes: 400,
        fingerprint: "fingerprint-b".to_string(),
    };
    let report = build_core_performance_regression_report(
        &CorePerformanceRegressionConfig {
            max_allowed_official_row_drop: 0,
            max_allowed_outcome_link_drop: 0,
            max_allowed_counterfactual_drop: 0,
            max_allowed_calibration_worsening: 0.01,
            max_allowed_denial_rate_increase: 0.05,
            max_allowed_actionability_drop: 0.10,
            max_allowed_storage_growth_bytes: 10,
            ..CorePerformanceRegressionConfig::default()
        },
        Some(previous.clone()),
        current.clone(),
    );
    assert!(report.regression_detected);
    assert!(
        report
            .regressions
            .iter()
            .any(|item| item.contains("official_row_drop"))
    );
    assert!(
        report
            .regressions
            .iter()
            .any(|item| item.contains("outcome_link_drop"))
    );
    assert!(
        report
            .regressions
            .iter()
            .any(|item| item.contains("counterfactual_drop"))
    );
    assert!(
        report
            .regressions
            .iter()
            .any(|item| item.contains("calibration_worsened"))
    );
    assert!(
        report
            .regressions
            .iter()
            .any(|item| item.contains("denial_rate_increase"))
    );
    assert!(
        report
            .regressions
            .iter()
            .any(|item| item.contains("actionability_drop"))
    );
    assert!(
        report
            .regressions
            .iter()
            .any(|item| item.contains("storage_growth"))
    );

    let current_path =
        common::output_dir("core-performance-regression-from-config").join("current_summary.json");
    current
        .to_json_path(&current_path)
        .expect("write current summary");
    let from_config =
        CorePerformanceRegressionReport::from_config(&CorePerformanceRegressionConfig {
            current_scorecard_path: Some(current_path.display().to_string()),
            previous_scorecard_path: None,
            ..CorePerformanceRegressionConfig::default()
        })
        .expect("from config without previous");
    assert!(!from_config.comparable);

    let missing_auth = build_core_bottleneck_report(&CoreBottleneckInputs {
        provider_auth_missing: true,
        ..CoreBottleneckInputs::default()
    });
    assert_eq!(
        missing_auth.primary_bottleneck,
        CoreBottleneckKind::MissingOfficialAuth
    );
    assert_eq!(
        missing_auth.recommended_next_action,
        CoreBottleneckRecommendation::OfficialProviderAuthFirst
    );

    let outcome_links = build_core_bottleneck_report(&CoreBottleneckInputs {
        outcome_links_missing: true,
        ..CoreBottleneckInputs::default()
    });
    assert_eq!(
        outcome_links.recommended_next_action,
        CoreBottleneckRecommendation::ImproveOutcomeLinkingFirst
    );

    let baseline = build_core_bottleneck_report(&CoreBottleneckInputs {
        baseline_references_missing: true,
        ..CoreBottleneckInputs::default()
    });
    assert_eq!(
        baseline.recommended_next_action,
        CoreBottleneckRecommendation::ImproveBaselineReferenceDepth
    );

    let counterfactuals = build_core_bottleneck_report(&CoreBottleneckInputs {
        no_trade_counterfactuals_missing: true,
        ..CoreBottleneckInputs::default()
    });
    assert_eq!(
        counterfactuals.recommended_next_action,
        CoreBottleneckRecommendation::ImproveCounterfactualDepthFirst
    );

    let risk = build_core_bottleneck_report(&CoreBottleneckInputs {
        risk_overblocking: true,
        ..CoreBottleneckInputs::default()
    });
    assert_eq!(
        risk.recommended_next_action,
        CoreBottleneckRecommendation::ImproveRiskGovernorFirst
    );

    let calibration = build_core_bottleneck_report(&CoreBottleneckInputs {
        poor_calibration: true,
        ..CoreBottleneckInputs::default()
    });
    assert_eq!(
        calibration.recommended_next_action,
        CoreBottleneckRecommendation::ImproveSignalModelFirst
    );

    let budget = build_core_bottleneck_report(&CoreBottleneckInputs {
        storage_budget_exceeded: true,
        ..CoreBottleneckInputs::default()
    });
    assert_eq!(
        budget.recommended_next_action,
        CoreBottleneckRecommendation::HoldCurrentScope
    );

    let none = build_core_bottleneck_report(&CoreBottleneckInputs::default());
    assert_eq!(
        none.primary_bottleneck,
        CoreBottleneckKind::NoBottleneckDetected
    );
    assert_eq!(none.to_text(), none.to_text());

    let healthy = CorePerformanceFinalStatus::CorePerformanceHealthyForResearch;
    assert_ne!(healthy, CorePerformanceFinalStatus::CoreDiagnosticOnly);
}

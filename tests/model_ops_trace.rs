mod common;
#[path = "support/sprint68_support.rs"]
mod sprint68_support;

use soma_zero::{
    ModelCardTraceGraphStatus, ModelOpsTraceConfig, ModelOpsTraceRunner, ModelVersionDiffStatus,
    TraceArtifactKind, TraceEdgeKind, TraceNodeKind,
};

#[test]
fn trace_config_defaults_are_local_only_and_static() {
    let config = ModelOpsTraceConfig::default();
    assert!(config.require_local_paths);
    assert!(config.generate_html_fragments);
    let encoded = toml::to_string(&config).expect("serialize trace config");
    for forbidden in [
        "broker_",
        "order_",
        "account_",
        "live_",
        "runtime_",
        "training_",
    ] {
        assert!(
            !encoded.contains(forbidden),
            "unexpected forbidden config field: {forbidden}"
        );
    }

    let mut bad = config.clone();
    bad.model_ops_rollup_paths = vec!["https://example.com/rollup.json".to_string()];
    assert!(bad.validate().is_err());
}

#[test]
fn trace_limits_are_enforced() {
    let mut config = sprint68_support::trace_config_from_example(
        "soma_model_ops_trace.toml",
        "trace-limits-models",
    );
    config.max_models = 1;
    assert!(ModelOpsTraceRunner::default().run(&config).is_err());

    let mut config = sprint68_support::trace_config_from_example(
        "soma_model_ops_trace.toml",
        "trace-limits-versions",
    );
    config.max_versions = 2;
    assert!(ModelOpsTraceRunner::default().run(&config).is_err());

    let mut config = sprint68_support::trace_config_from_example(
        "soma_model_ops_trace.toml",
        "trace-limits-edges",
    );
    config.max_trace_edges = 1;
    assert!(ModelOpsTraceRunner::default().run(&config).is_err());

    let mut config = sprint68_support::trace_config_from_example(
        "soma_model_ops_trace.toml",
        "trace-limits-artifacts",
    );
    config.max_artifacts = 1;
    assert!(ModelOpsTraceRunner::default().run(&config).is_err());

    let mut config = sprint68_support::trace_config_from_example(
        "soma_model_ops_trace.toml",
        "trace-limits-bytes",
    );
    config.max_bytes = 64;
    assert!(ModelOpsTraceRunner::default().run(&config).is_err());
}

#[test]
fn artifact_index_and_trace_graph_cover_core_evidence() {
    let bundle = sprint68_support::run_trace("soma_model_ops_trace.toml", "trace-core");
    let kinds = bundle
        .artifact_trace_index
        .artifacts
        .iter()
        .map(|item| item.artifact_kind)
        .collect::<std::collections::BTreeSet<_>>();
    for required in [
        TraceArtifactKind::ModelVersionSummaryCard,
        TraceArtifactKind::RegressionExplanation,
        TraceArtifactKind::OperatorQARollup,
        TraceArtifactKind::DecisionLogRollup,
        TraceArtifactKind::RiskRollup,
        TraceArtifactKind::ActionPriority,
        TraceArtifactKind::BaselineSnapshot,
        TraceArtifactKind::CurrentSnapshot,
    ] {
        assert!(
            kinds.contains(&required),
            "missing artifact kind {:?}",
            required
        );
    }

    let graph = bundle
        .model_card_trace_graphs
        .iter()
        .find(|item| item.model_id == "ext-model-b" && item.model_version == "1.0.0")
        .expect("ext-model-b trace graph");
    assert_eq!(graph.graph_status, ModelCardTraceGraphStatus::TraceReady);
    let node_kinds = graph
        .nodes
        .iter()
        .map(|item| item.node_kind)
        .collect::<std::collections::BTreeSet<_>>();
    for required in [
        TraceNodeKind::ModelVersionCard,
        TraceNodeKind::Decision,
        TraceNodeKind::Regression,
        TraceNodeKind::QAItem,
        TraceNodeKind::RiskBand,
        TraceNodeKind::ActionPriority,
        TraceNodeKind::Artifact,
        TraceNodeKind::MambaDeferredState,
    ] {
        assert!(
            node_kinds.contains(&required),
            "missing node kind {:?}",
            required
        );
    }
    let edge_kinds = graph
        .edges
        .iter()
        .map(|item| item.edge_kind)
        .collect::<std::collections::BTreeSet<_>>();
    for required in [
        TraceEdgeKind::DerivedFrom,
        TraceEdgeKind::CausedBy,
        TraceEdgeKind::Blocks,
        TraceEdgeKind::Recommends,
        TraceEdgeKind::NeedsReview,
    ] {
        assert!(
            edge_kinds.contains(&required),
            "missing edge kind {:?}",
            required
        );
    }
}

#[test]
fn model_version_diff_trace_detects_changed_and_missing_targets() {
    let bundle = sprint68_support::run_trace("soma_model_ops_trace.toml", "trace-diff");
    let ext_model_b = bundle
        .model_version_diff_trace_report
        .diffs
        .iter()
        .find(|item| item.model_id == "ext-model-b" && item.model_version == "1.0.0")
        .expect("ext-model-b diff");
    assert_eq!(
        ext_model_b.diff_status,
        ModelVersionDiffStatus::UnexpectedDiff
    );
    assert_eq!(
        ext_model_b.changed_risk_status.as_deref(),
        Some("Low -> Critical")
    );
    assert_eq!(
        ext_model_b.changed_leaderboard_status.as_deref(),
        Some("NoChange -> NewlyBlocked")
    );
    assert!(
        ext_model_b
            .added_artifacts
            .contains(&"regression_guard".to_string())
    );
    assert!(
        ext_model_b
            .removed_artifacts
            .contains(&"leaderboard".to_string())
    );

    let ext_model_a = bundle
        .model_version_diff_trace_report
        .diffs
        .iter()
        .find(|item| item.model_id == "ext-model-a" && item.model_version == "1.1.0")
        .expect("ext-model-a diff");
    assert!(
        ext_model_a
            .added_artifacts
            .contains(&"prediction_csv".to_string())
    );

    let missing_target = bundle
        .model_version_diff_trace_report
        .diffs
        .iter()
        .find(|item| item.model_id == "ext-model-a" && item.model_version == "1.2.0")
        .expect("missing target diff");
    assert_eq!(
        missing_target.diff_status,
        ModelVersionDiffStatus::MissingComparisonTarget
    );
}

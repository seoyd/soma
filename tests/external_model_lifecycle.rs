mod common;
#[path = "support/sprint65_support.rs"]
mod sprint65_support;

use soma_zero::{
    ExternalModelArtifactKind, ExternalModelArtifactRegistry, ExternalModelLifecycleStatus,
    ExternalModelRegistryEntry, ExternalModelResearchOpsRunner,
};

fn sample_entry(
    registry: &ExternalModelArtifactRegistry,
    artifact_kind: ExternalModelArtifactKind,
    model_id: &str,
    model_version: &str,
) -> ExternalModelRegistryEntry {
    let mut entry = registry
        .entries
        .iter()
        .find(|entry| entry.artifact_kind == artifact_kind)
        .expect("find sample registry entry")
        .clone();
    entry.entry_id = format!("{artifact_kind:?}:{model_id}:{model_version}");
    entry.model_id = model_id.to_string();
    entry.model_version = model_version.to_string();
    entry
}

#[test]
fn registered_imported_and_evaluated_transitions_are_allowed() {
    let mut config = sprint65_support::research_ops_config_from_example(
        "soma_external_model_research_ops.toml",
        "lifecycle-early-states",
    );
    let mut registry: ExternalModelArtifactRegistry =
        sprint65_support::read_json(&config.external_artifact_registry_paths[0]);
    let model_card = sample_entry(
        &registry,
        ExternalModelArtifactKind::ModelCard,
        "registered-model",
        "0.1.0",
    );
    let import_report = sample_entry(
        &registry,
        ExternalModelArtifactKind::ImportReport,
        "imported-model",
        "0.2.0",
    );
    let evaluation_report = sample_entry(
        &registry,
        ExternalModelArtifactKind::EvaluationReport,
        "evaluated-model",
        "0.3.0",
    );
    registry.entries.extend([
        model_card,
        sample_entry(
            &registry,
            ExternalModelArtifactKind::ModelCard,
            "imported-model",
            "0.2.0",
        ),
        import_report,
        sample_entry(
            &registry,
            ExternalModelArtifactKind::ModelCard,
            "evaluated-model",
            "0.3.0",
        ),
        sample_entry(
            &registry,
            ExternalModelArtifactKind::ImportReport,
            "evaluated-model",
            "0.3.0",
        ),
        evaluation_report,
    ]);
    config.external_artifact_registry_paths[0] =
        sprint65_support::write_support_json("lifecycle-early-states", "registry.json", &registry);
    config.conservative_leaderboard_paths = Vec::new();
    config.owner_model_review_paths = Vec::new();

    let bundle = ExternalModelResearchOpsRunner::default()
        .run(&config)
        .expect("run lifecycle early states");
    let registered = bundle
        .lifecycle_records
        .iter()
        .find(|record| record.model_id == "registered-model")
        .expect("registered model record");
    assert_eq!(
        registered.current_status,
        ExternalModelLifecycleStatus::Registered
    );
    assert!(
        registered
            .allowed_transitions
            .contains(&"Imported".to_string())
    );

    let imported = bundle
        .lifecycle_records
        .iter()
        .find(|record| record.model_id == "imported-model")
        .expect("imported model record");
    assert_eq!(
        imported.current_status,
        ExternalModelLifecycleStatus::Imported
    );
    assert!(
        imported
            .allowed_transitions
            .contains(&"Evaluated".to_string())
    );

    let evaluated = bundle
        .lifecycle_records
        .iter()
        .find(|record| record.model_id == "evaluated-model")
        .expect("evaluated model record");
    assert_eq!(
        evaluated.current_status,
        ExternalModelLifecycleStatus::Evaluated
    );
    assert!(
        evaluated
            .allowed_transitions
            .contains(&"ResearchCandidate".to_string())
    );
}

#[test]
fn research_candidate_transitions_and_forbidden_runtime_paths_are_explicit() {
    let mut config = sprint65_support::research_ops_config_from_example(
        "soma_external_model_research_ops.toml",
        "lifecycle-research-candidate",
    );
    config.owner_model_review_paths = vec![sprint65_support::write_support_json(
        "lifecycle-research-candidate",
        "owner_actions.json",
        &Vec::<serde_json::Value>::new(),
    )];

    let bundle = ExternalModelResearchOpsRunner::default()
        .run(&config)
        .expect("run lifecycle candidate");
    let candidate = bundle
        .lifecycle_records
        .iter()
        .find(|record| record.model_id == "ext-model-a" && record.model_version == "1.1.0")
        .expect("research candidate record");
    assert_eq!(
        candidate.current_status,
        ExternalModelLifecycleStatus::ResearchCandidate
    );
    assert!(
        candidate
            .allowed_transitions
            .contains(&"Watchlisted".to_string())
    );
    assert!(
        candidate
            .allowed_transitions
            .contains(&"DiagnosticOnly".to_string())
    );
    assert!(
        candidate
            .allowed_transitions
            .contains(&"Retired".to_string())
    );
    for forbidden in ["Live", "RuntimeIntegrated", "BrokerExecutable"] {
        assert!(
            candidate
                .forbidden_transitions
                .contains(&forbidden.to_string())
        );
    }
}

#[test]
fn lifecycle_output_is_deterministic() {
    let first = sprint65_support::research_ops_config_from_example(
        "soma_external_model_research_ops.toml",
        "lifecycle-determinism-first",
    );
    let second = sprint65_support::research_ops_config_from_example(
        "soma_external_model_research_ops.toml",
        "lifecycle-determinism-second",
    );
    let first_bundle = ExternalModelResearchOpsRunner::default()
        .run(&first)
        .expect("run first lifecycle bundle");
    let second_bundle = ExternalModelResearchOpsRunner::default()
        .run(&second)
        .expect("run second lifecycle bundle");
    assert_eq!(
        first_bundle.lifecycle_records,
        second_bundle.lifecycle_records
    );
}

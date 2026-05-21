mod common;

use soma_zero::{
    CommitteeArtifactKind, CommitteeBenchmarkConfig, CommitteeBenchmarkRunner,
    CommitteeMaterializationConfig, ReasonCode,
};

#[test]
fn same_fixture_input_produces_same_committee_benchmark_output() {
    let materialization_path =
        common::output_dir("benchmark-determinism-mat").join("materialize.toml");
    std::fs::write(
        &materialization_path,
        CommitteeMaterializationConfig {
            materialization_id: "benchmark-determinism".to_string(),
            allowed_artifact_kinds: vec![CommitteeArtifactKind::FixtureScenario],
            output_root: common::output_dir("benchmark-determinism-out")
                .display()
                .to_string(),
            reason_codes: vec![ReasonCode::CommitteeMaterializationBuilt],
            ..CommitteeMaterializationConfig::default()
        }
        .to_toml_string()
        .expect("toml"),
    )
    .expect("write");
    let cfg = CommitteeBenchmarkConfig {
        benchmark_id: "benchmark-determinism".to_string(),
        materialization_config_path: Some(materialization_path.display().to_string()),
        output_root: common::output_dir("benchmark-determinism-run")
            .display()
            .to_string(),
        require_core_check: false,
        ..CommitteeBenchmarkConfig::default()
    };
    let first = CommitteeBenchmarkRunner::default()
        .run(&cfg)
        .expect("first");
    let second = CommitteeBenchmarkRunner::default()
        .run(&cfg)
        .expect("second");
    assert_eq!(first.audit_summary, second.audit_summary);
    assert_eq!(first.to_text(), second.to_text());
}

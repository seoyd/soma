use std::path::Path;

use soma_zero::{
    ControlTowerV1Builder, ControlTowerV1Config, DashboardV1Renderer,
    TrinityCommitteeOperationalLoopConfig, TrinityOperationalLoopRunner,
};

#[test]
fn control_tower_loads_operational_loop_panels() {
    let loop_config = TrinityCommitteeOperationalLoopConfig::from_toml_path(Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/examples/soma_trinity_operational_loop_kis.toml"
    )))
    .unwrap();
    let bundle = TrinityOperationalLoopRunner::default()
        .run(&loop_config)
        .unwrap();
    let report_path = loop_config
        .artifact_dir()
        .join("trinity_operational_loop_report.json");
    let config = ControlTowerV1Config {
        control_tower_id: "sprint56-control-tower".to_string(),
        operational_loop_report_paths: vec![report_path.display().to_string()],
        output_root: "target/test-sprint56-control-tower".to_string(),
        render_html: true,
        render_json: true,
        render_text: true,
        ..ControlTowerV1Config::default()
    };
    let state = ControlTowerV1Builder::default()
        .build(&config, None)
        .unwrap();
    assert_eq!(
        state.operational_loop_panel.generated_candidates,
        bundle.report.generated_candidate_count
    );
    assert_eq!(state.trinity_status_panel.active_count, 3);
    assert!(!state.candidate_lifecycle_panel.candidate_views.is_empty());
    let render_report = DashboardV1Renderer::default()
        .render(&state, &config)
        .unwrap();
    assert_eq!(render_report.panel_count, 17);
}

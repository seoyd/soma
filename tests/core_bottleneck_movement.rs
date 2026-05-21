use soma_zero::{
    CoreBottleneckKind, CoreBottleneckMovementKind, build_core_bottleneck_movement_report,
};

#[test]
fn movement_report_detects_no_movement() {
    let report = build_core_bottleneck_movement_report(
        Some(CoreBottleneckKind::ScenarioMaterializationWeak),
        Some(CoreBottleneckKind::ScenarioMaterializationWeak),
    );
    assert_eq!(report.movement_kind, CoreBottleneckMovementKind::NoMovement);
}

#[test]
fn movement_report_detects_shift_from_materialization() {
    let report = build_core_bottleneck_movement_report(
        Some(CoreBottleneckKind::ScenarioMaterializationWeak),
        Some(CoreBottleneckKind::MissingOutcomeLinks),
    );
    assert_eq!(
        report.movement_kind,
        CoreBottleneckMovementKind::MovedToOutcomeLinking
    );
}

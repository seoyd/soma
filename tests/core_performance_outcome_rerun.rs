use soma_zero::{
    CoreBottleneckKind, CorePerformanceFinalStatus, CorePerformanceRerunAfterOutcomeLinkage,
};

#[test]
fn core_performance_rerun_marks_evidence_too_weak_movement() {
    let summary = CorePerformanceRerunAfterOutcomeLinkage::build(
        Some(CoreBottleneckKind::EvidenceTooWeak),
        Some(CoreBottleneckKind::RiskOverBlocking),
        Some(CorePerformanceFinalStatus::CoreNeedsMoreEvidence),
        Some(CorePerformanceFinalStatus::CoreBlockedByRiskBehavior),
        true,
        Vec::new(),
    );
    assert!(summary.ran);
    assert!(summary.bottleneck_changed);
    assert!(summary.bottleneck_moved_from_evidence_too_weak);
    assert!(summary.status_improved);
}

#[test]
fn core_performance_rerun_missing_emits_warning() {
    let summary = CorePerformanceRerunAfterOutcomeLinkage::missing("not possible");
    assert!(!summary.ran);
    assert_eq!(summary.warnings, vec!["not possible".to_string()]);
}

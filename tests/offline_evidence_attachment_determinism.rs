#[path = "support/sprint69_support.rs"]
mod support;

use std::fs;

#[test]
fn offline_evidence_attachment_outputs_are_deterministic() {
    let first = support::run_offline_attachment(
        "soma_offline_evidence_attach.toml",
        "offline-evidence-attachment-determinism-a",
    );
    let second = support::run_offline_attachment(
        "soma_offline_evidence_attach.toml",
        "offline-evidence-attachment-determinism-b",
    );

    assert_eq!(
        first.offline_evidence_attachment_registry,
        second.offline_evidence_attachment_registry
    );
    assert_eq!(
        first.prediction_history_expansion_report,
        second.prediction_history_expansion_report
    );
    assert_eq!(
        first.retirement_regression_evidence_pack,
        second.retirement_regression_evidence_pack
    );
    assert_eq!(
        first.evidence_gap_closure_v2_report,
        second.evidence_gap_closure_v2_report
    );
    assert_eq!(
        first.owner_checklist_closure_report,
        second.owner_checklist_closure_report
    );
    assert_eq!(
        first.direct_watch_readiness_score_v2,
        second.direct_watch_readiness_score_v2
    );
    assert_eq!(
        first.operator_briefing_readiness_gate,
        second.operator_briefing_readiness_gate
    );

    let first_html = fs::read_to_string(
        support::attachment_output_path("offline-evidence-attachment-determinism-a")
            .join("sprint72-offline-evidence-attachment")
            .join("fragments")
            .join("direct_watch_readiness.html"),
    )
    .expect("first html");
    let second_html = fs::read_to_string(
        support::attachment_output_path("offline-evidence-attachment-determinism-b")
            .join("sprint72-offline-evidence-attachment")
            .join("fragments")
            .join("direct_watch_readiness.html"),
    )
    .expect("second html");
    assert_eq!(first_html, second_html);

    let first_summary = fs::read_to_string(
        support::attachment_output_path("offline-evidence-attachment-determinism-a")
            .join("sprint72-offline-evidence-attachment")
            .join("summary.txt"),
    )
    .expect("first summary");
    let second_summary = fs::read_to_string(
        support::attachment_output_path("offline-evidence-attachment-determinism-b")
            .join("sprint72-offline-evidence-attachment")
            .join("summary.txt"),
    )
    .expect("second summary");
    assert_eq!(first_summary, second_summary);
}

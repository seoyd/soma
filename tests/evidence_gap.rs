use std::path::PathBuf;

use soma_zero::{
    Sprint14Runner, Sprint14Track, Sprint14TrackSpecificReport, evidence_gap_report_to_text,
};

fn report_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("soma_ablations")
        .join("ablation_feature_lab")
        .join("ablation_report.json")
}

fn sprint14_report() -> soma_zero::Sprint14Report {
    let path = report_path();
    if !path.exists() {
        let status = std::process::Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
            .args([
                "ablation",
                "--config",
                "examples/soma_ablation_feature_lab.toml",
            ])
            .status()
            .expect("run ablation");
        assert!(status.success());
    }
    Sprint14Runner::default()
        .run_from_ablation_report_path(&path)
        .expect("sprint14 report")
}

#[test]
fn evidence_gap_report_is_deterministic() {
    let report_a = sprint14_report();
    let report_b = sprint14_report();
    let Sprint14TrackSpecificReport::NeedMoreExperiments(gap_a) = report_a.track_specific_report;
    let Sprint14TrackSpecificReport::NeedMoreExperiments(gap_b) = report_b.track_specific_report;
    assert_eq!(gap_a, gap_b);
    assert_eq!(
        evidence_gap_report_to_text(&gap_a),
        evidence_gap_report_to_text(&gap_b)
    );
}

#[test]
fn minimum_evidence_plan_is_generated() {
    let report = sprint14_report();
    let Sprint14TrackSpecificReport::NeedMoreExperiments(gap) = report.track_specific_report;
    assert!(gap.minimum_evidence_plan.additional_usable_datasets_needed > 0);
    assert!(gap.minimum_evidence_plan.additional_outcome_records_needed > 0);
}

#[test]
fn insufficient_evidence_blocks_expansion() {
    let report = sprint14_report();
    assert_eq!(
        report.decision_record.selected_track,
        Sprint14Track::NeedMoreExperiments
    );
    let Sprint14TrackSpecificReport::NeedMoreExperiments(gap) = report.track_specific_report;
    assert!(gap.insufficient_evidence);
    assert!(gap.minimum_evidence_plan.blocked_expansion);
}

#[test]
fn need_more_experiments_track_does_not_change_runtime_logic() {
    for path in [
        "src/data/diagnostics.rs",
        "src/feature/diagnostics.rs",
        "src/risk/diagnostics.rs",
        "src/regime/diagnostics.rs",
        "src/signal/diagnostics.rs",
        "src/league/design_review.rs",
    ] {
        assert!(
            !PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join(path)
                .exists(),
            "unexpected unselected-track implementation at {path}"
        );
    }
}

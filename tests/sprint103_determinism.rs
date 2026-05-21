mod support;

use support::sprint103_support::run_sprint103;

#[test]
fn sprint103_bundle_is_deterministic_and_summary_has_required_headings() {
    let mut left = run_sprint103(
        "soma_sprint103_paper_rotation_close.toml",
        "sprint103-determinism-left",
    );
    let mut right = run_sprint103(
        "soma_sprint103_paper_rotation_close.toml",
        "sprint103-determinism-right",
    );
    left.storage_report.output_dir = "<normalized>".to_string();
    right.storage_report.output_dir = "<normalized>".to_string();
    assert_eq!(
        left.to_json_string().expect("left json"),
        right.to_json_string().expect("right json")
    );
    for heading in [
        "## 1. Sprint summary",
        "## 19. Risk Governor handoff warning closure v2",
        "## 34. Control Tower paper rotation closure panel",
        "## 49. Next gstack sprint recommendation",
    ] {
        assert!(
            left.final_summary.contains(heading),
            "missing heading {heading}"
        );
    }
}

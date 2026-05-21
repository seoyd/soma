mod support;

use soma_zero::FifthPatchResumeGateV5;
use support::sprint115_support::{read_fixture, run_sprint115};

#[test]
fn fifth_patch_resume_gate_v5_remains_blocked() {
    let bundle = run_sprint115(
        "soma_fifth_patch_resume_gate_v5.toml",
        "fifth-patch-resume-gate-v5",
    );
    let expected: FifthPatchResumeGateV5 =
        read_fixture("sprint115_data/fifth_patch_resume_gate_expected.json");
    assert_eq!(bundle.fifth_patch_resume_gate_v5, expected);
    assert!(
        !bundle
            .fifth_patch_resume_gate_v5
            .resume_allowed_for_later_sprint
    );
}

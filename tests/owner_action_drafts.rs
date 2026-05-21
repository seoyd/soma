mod common;

use std::fs;

use soma_zero::{
    ControlTowerV1Builder, ControlTowerV1Config, OwnerActionDraftKind,
    generate_owner_action_draft_bundle,
};

#[test]
fn owner_action_drafts_are_generated_without_auto_apply() {
    let mut config = ControlTowerV1Config::from_toml_path(&common::example_path(
        "soma_dashboard_action_drafts.toml",
    ))
    .expect("config");
    config.output_root = common::sprint54_output_dir("owner-action-drafts")
        .display()
        .to_string();
    let state = ControlTowerV1Builder::default()
        .build(
            &config,
            Some(&common::example_path("soma_dashboard_action_drafts.toml")),
        )
        .expect("state");
    let bundle = generate_owner_action_draft_bundle(&state, &config).expect("bundle");
    assert!(
        bundle
            .drafts
            .iter()
            .any(|draft| matches!(draft.draft_kind, OwnerActionDraftKind::NoteDraft))
    );
    assert!(
        bundle
            .drafts
            .iter()
            .any(|draft| matches!(draft.draft_kind, OwnerActionDraftKind::HoldDraft))
    );
    assert!(
        bundle
            .drafts
            .iter()
            .any(|draft| matches!(draft.draft_kind, OwnerActionDraftKind::DismissDraft))
    );
    assert!(
        bundle
            .drafts
            .iter()
            .any(|draft| matches!(draft.draft_kind, OwnerActionDraftKind::ReanalysisDraft))
    );
    assert!(
        bundle
            .drafts
            .iter()
            .any(|draft| matches!(draft.draft_kind, OwnerActionDraftKind::ThesisNoteDraft))
    );
    assert!(bundle.drafts.iter().any(|draft| matches!(
        draft.draft_kind,
        OwnerActionDraftKind::PaperConfirmDraft
    ) && draft.target_candidate_id.as_deref()
        == Some("cand-101")));
    assert!(!bundle.drafts.iter().any(|draft| matches!(
        draft.draft_kind,
        OwnerActionDraftKind::PaperConfirmDraft
    ) && draft.target_candidate_id.as_deref()
        == Some("cand-102")));
    assert!(!bundle.drafts.iter().any(|draft| matches!(
        draft.draft_kind,
        OwnerActionDraftKind::PaperConfirmDraft
    ) && draft.target_candidate_id.as_deref()
        == Some("cand-103")));
    let first_path = &bundle
        .drafts
        .first()
        .expect("draft")
        .suggested_owner_input_config_path;
    let contents = fs::read_to_string(first_path).expect("draft file");
    assert!(contents.contains("apply_id") || contents.contains("thesis_notes_path"));
    assert_eq!(bundle.drafts, {
        let mut copy = bundle.drafts.clone();
        copy.sort_by(|left, right| left.draft_id.cmp(&right.draft_id));
        copy
    });
}

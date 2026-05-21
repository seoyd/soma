mod common;
#[path = "support/sprint64_support.rs"]
mod sprint64_support;

use soma_zero::{
    ConservativeExternalLeaderboardStatus, ExternalArtifactRegistryRunner, LeaderboardEntryStatus,
};

#[test]
fn leaderboard_ranks_eligible_entries_and_keeps_reference_rows_visible() {
    let config = sprint64_support::registry_config_from_example(
        "soma_conservative_external_leaderboard.toml",
        "leaderboard-default",
    );
    let leaderboard = ExternalArtifactRegistryRunner::default()
        .run_leaderboard(&config)
        .expect("run leaderboard");
    assert_eq!(
        leaderboard.leaderboard_status,
        ConservativeExternalLeaderboardStatus::LeaderboardReady
    );
    assert_eq!(leaderboard.eligible_entries, 2);
    assert_eq!(leaderboard.blocked_entries, 1);
    assert!(leaderboard.baseline_entry.is_some());
    assert!(leaderboard.trinity_entry.is_some());
    assert!(leaderboard.no_trade_entry.is_some());
    assert!(leaderboard.risk_denied_entry.is_some());
    assert_eq!(leaderboard.entries[0].rank, Some(1));
}

#[test]
fn missing_model_card_blocks_leaderboard_inclusion() {
    let mut config = sprint64_support::registry_config_from_example(
        "soma_conservative_external_leaderboard.toml",
        "leaderboard-missing-card",
    );
    config.external_model_card_paths.remove(0);

    let leaderboard = ExternalArtifactRegistryRunner::default()
        .run_leaderboard(&config)
        .expect("run missing card leaderboard");
    assert!(
        leaderboard
            .entries
            .iter()
            .any(|entry| entry.entry_status == LeaderboardEntryStatus::BlockedByModelCard)
    );
}

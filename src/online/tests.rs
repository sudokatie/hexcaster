// Integration tests for online module

use super::*;

#[test]
#[ignore = "requires running server"]
fn test_full_workflow() {
    // Create enabled config
    let config = OnlineConfig {
        enabled: true,
        server_url: "https://hexcaster.blackabee.com/api".to_string(),
        player_name: Some("IntegrationTest".to_string()),
        signature_secret: Some("test_secret".to_string()),
    };

    let client = LeaderboardClient::new(config);

    // Fetch leaderboard
    let leaderboard = client.fetch_leaderboard(LeaderboardPeriod::AllTime, 10);
    assert!(leaderboard.is_ok());

    // Submit score
    let submit = client.submit_score(500, 3, 75, 99999);
    assert!(submit.is_ok());
    assert!(submit.unwrap().success);
}

#[test]
fn test_period_serialization() {
    use serde_json;

    let daily = LeaderboardPeriod::Daily;
    let json = serde_json::to_string(&daily).unwrap();
    assert_eq!(json, r#""daily""#);

    let parsed: LeaderboardPeriod = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, LeaderboardPeriod::Daily);
}

#[test]
fn test_entry_serialization() {
    use serde_json;
    use chrono::Utc;

    let entry = LeaderboardEntry {
        rank: 1,
        player_name: "Champion".to_string(),
        score: 10000,
        floor_reached: 10,
        turns_taken: 500,
        submitted_at: Utc::now(),
    };

    let json = serde_json::to_string(&entry).unwrap();
    assert!(json.contains("Champion"));
    assert!(json.contains("10000"));

    let parsed: LeaderboardEntry = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.rank, 1);
    assert_eq!(parsed.player_name, "Champion");
}

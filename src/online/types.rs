// Online leaderboard types

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Time period for leaderboard filtering
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LeaderboardPeriod {
    Daily,
    Weekly,
    AllTime,
}

impl Default for LeaderboardPeriod {
    fn default() -> Self {
        Self::AllTime
    }
}

impl std::fmt::Display for LeaderboardPeriod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Daily => write!(f, "daily"),
            Self::Weekly => write!(f, "weekly"),
            Self::AllTime => write!(f, "all-time"),
        }
    }
}

/// A score entry in the leaderboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaderboardEntry {
    pub rank: u32,
    pub player_name: String,
    pub score: u32,
    pub floor_reached: u32,
    pub turns_taken: u32,
    pub submitted_at: DateTime<Utc>,
}

/// Score submission for anti-cheat verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreSubmission {
    pub player_name: String,
    pub score: u32,
    pub floor_reached: u32,
    pub turns_taken: u32,
    pub game_seed: u64,
    pub game_version: String,
    pub signature: String,
}

impl ScoreSubmission {
    /// Create a new score submission
    pub fn new(
        player_name: String,
        score: u32,
        floor_reached: u32,
        turns_taken: u32,
        game_seed: u64,
    ) -> Self {
        Self {
            player_name,
            score,
            floor_reached,
            turns_taken,
            game_seed,
            game_version: env!("CARGO_PKG_VERSION").to_string(),
            signature: String::new(), // Will be computed
        }
    }

    /// Sign the submission for anti-cheat verification
    /// Uses HMAC-SHA256 with a secret key
    pub fn sign(&mut self, secret: &[u8]) {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        // Simple signature for now - would use proper HMAC in production
        let mut hasher = DefaultHasher::new();
        self.player_name.hash(&mut hasher);
        self.score.hash(&mut hasher);
        self.floor_reached.hash(&mut hasher);
        self.turns_taken.hash(&mut hasher);
        self.game_seed.hash(&mut hasher);
        self.game_version.hash(&mut hasher);
        secret.hash(&mut hasher);
        
        self.signature = format!("{:016x}", hasher.finish());
    }

    /// Verify the submission signature
    pub fn verify(&self, secret: &[u8]) -> bool {
        let mut check = self.clone();
        check.signature = String::new();
        check.sign(secret);
        check.signature == self.signature
    }
}

/// Response from leaderboard fetch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaderboardResponse {
    pub period: LeaderboardPeriod,
    pub entries: Vec<LeaderboardEntry>,
    pub total_players: u32,
    pub your_rank: Option<u32>,
}

/// Response from score submission
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitResponse {
    pub success: bool,
    pub rank: Option<u32>,
    pub message: String,
}

/// Online leaderboard configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnlineConfig {
    pub enabled: bool,
    pub server_url: String,
    pub player_name: Option<String>,
}

impl Default for OnlineConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            server_url: "https://hexcaster.blackabee.com/api".to_string(),
            player_name: None,
        }
    }
}

#[cfg(test)]
mod type_tests {
    use super::*;

    #[test]
    fn test_period_display() {
        assert_eq!(LeaderboardPeriod::Daily.to_string(), "daily");
        assert_eq!(LeaderboardPeriod::Weekly.to_string(), "weekly");
        assert_eq!(LeaderboardPeriod::AllTime.to_string(), "all-time");
    }

    #[test]
    fn test_period_default() {
        assert_eq!(LeaderboardPeriod::default(), LeaderboardPeriod::AllTime);
    }

    #[test]
    fn test_score_submission_new() {
        let sub = ScoreSubmission::new(
            "TestPlayer".to_string(),
            1000,
            5,
            100,
            12345,
        );
        assert_eq!(sub.player_name, "TestPlayer");
        assert_eq!(sub.score, 1000);
        assert_eq!(sub.floor_reached, 5);
        assert!(sub.signature.is_empty());
    }

    #[test]
    fn test_score_submission_sign_verify() {
        let secret = b"test_secret_key";
        let mut sub = ScoreSubmission::new(
            "Player1".to_string(),
            500,
            3,
            50,
            99999,
        );
        
        sub.sign(secret);
        assert!(!sub.signature.is_empty());
        assert!(sub.verify(secret));
        
        // Wrong secret should fail
        assert!(!sub.verify(b"wrong_secret"));
    }

    #[test]
    fn test_score_submission_tamper_detection() {
        let secret = b"test_secret";
        let mut sub = ScoreSubmission::new(
            "Player1".to_string(),
            500,
            3,
            50,
            99999,
        );
        
        sub.sign(secret);
        
        // Tamper with score
        sub.score = 9999;
        assert!(!sub.verify(secret));
    }

    #[test]
    fn test_online_config_default() {
        let config = OnlineConfig::default();
        assert!(!config.enabled);
        assert!(config.server_url.contains("hexcaster"));
        assert!(config.player_name.is_none());
    }
}

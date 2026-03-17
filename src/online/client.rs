// Online leaderboard client

use crate::online::types::*;
use thiserror::Error;

/// Errors that can occur during online operations
#[derive(Error, Debug)]
pub enum OnlineError {
    #[error("online features are disabled")]
    Disabled,
    
    #[error("network error: {0}")]
    Network(String),
    
    #[error("server error: {0}")]
    Server(String),
    
    #[error("invalid response: {0}")]
    InvalidResponse(String),
    
    #[error("player name not configured")]
    NoPlayerName,
}

/// Client for online leaderboard operations
pub struct LeaderboardClient {
    config: OnlineConfig,
}

impl LeaderboardClient {
    /// Create a new client with the given configuration
    pub fn new(config: OnlineConfig) -> Self {
        Self { config }
    }

    /// Check if online features are enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Get the configured player name
    pub fn player_name(&self) -> Option<&str> {
        self.config.player_name.as_deref()
    }

    /// Fetch leaderboard entries for a given period
    #[cfg(feature = "online")]
    pub fn fetch_leaderboard(
        &self,
        period: LeaderboardPeriod,
        limit: usize,
    ) -> Result<LeaderboardResponse, OnlineError> {
        if !self.config.enabled {
            return Err(OnlineError::Disabled);
        }

        let url = format!(
            "{}/leaderboard/{}?limit={}",
            self.config.server_url,
            period,
            limit.min(100)
        );

        let response = ureq::get(&url)
            .timeout(std::time::Duration::from_secs(10))
            .call()
            .map_err(|e| OnlineError::Network(e.to_string()))?;

        if response.status() != 200 {
            return Err(OnlineError::Server(format!(
                "Server returned status {}",
                response.status()
            )));
        }

        let body = response
            .into_string()
            .map_err(|e| OnlineError::InvalidResponse(e.to_string()))?;

        serde_json::from_str(&body)
            .map_err(|e| OnlineError::InvalidResponse(e.to_string()))
    }

    /// Fetch leaderboard (offline stub when online feature disabled)
    #[cfg(not(feature = "online"))]
    pub fn fetch_leaderboard(
        &self,
        period: LeaderboardPeriod,
        _limit: usize,
    ) -> Result<LeaderboardResponse, OnlineError> {
        if !self.config.enabled {
            return Err(OnlineError::Disabled);
        }

        // Return empty mock response when online feature is disabled
        Ok(LeaderboardResponse {
            period,
            entries: vec![],
            total_players: 0,
            your_rank: None,
        })
    }

    /// Submit a score to the leaderboard
    #[cfg(feature = "online")]
    pub fn submit_score(
        &self,
        score: u32,
        floor_reached: u32,
        turns_taken: u32,
        game_seed: u64,
    ) -> Result<SubmitResponse, OnlineError> {
        if !self.config.enabled {
            return Err(OnlineError::Disabled);
        }

        let player_name = self.config.player_name.as_ref()
            .ok_or(OnlineError::NoPlayerName)?
            .clone();

        let mut submission = ScoreSubmission::new(
            player_name,
            score,
            floor_reached,
            turns_taken,
            game_seed,
        );

        // Sign the submission using configured secret or default
        let secret = self.config.signature_secret.as_deref()
            .unwrap_or("hexcaster_default_secret");
        submission.sign(secret.as_bytes());

        let url = format!("{}/submit", self.config.server_url);
        let body = serde_json::to_string(&submission)
            .map_err(|e| OnlineError::InvalidResponse(e.to_string()))?;

        let response = ureq::post(&url)
            .set("Content-Type", "application/json")
            .timeout(std::time::Duration::from_secs(10))
            .send_string(&body)
            .map_err(|e| OnlineError::Network(e.to_string()))?;

        if response.status() != 200 {
            return Err(OnlineError::Server(format!(
                "Server returned status {}",
                response.status()
            )));
        }

        let response_body = response
            .into_string()
            .map_err(|e| OnlineError::InvalidResponse(e.to_string()))?;

        serde_json::from_str(&response_body)
            .map_err(|e| OnlineError::InvalidResponse(e.to_string()))
    }

    /// Submit score (offline stub when online feature disabled)
    #[cfg(not(feature = "online"))]
    pub fn submit_score(
        &self,
        score: u32,
        floor_reached: u32,
        turns_taken: u32,
        game_seed: u64,
    ) -> Result<SubmitResponse, OnlineError> {
        if !self.config.enabled {
            return Err(OnlineError::Disabled);
        }

        let player_name = self.config.player_name.as_ref()
            .ok_or(OnlineError::NoPlayerName)?
            .clone();

        let mut submission = ScoreSubmission::new(
            player_name,
            score,
            floor_reached,
            turns_taken,
            game_seed,
        );

        let secret = b"placeholder_secret";
        submission.sign(secret);

        // Return mock success when online feature is disabled
        Ok(SubmitResponse {
            success: true,
            rank: Some(1),
            message: "Score submitted (offline mode)".to_string(),
        })
    }

    /// Build the URL for a leaderboard endpoint
    #[allow(dead_code)]
    fn leaderboard_url(&self, period: LeaderboardPeriod) -> String {
        format!("{}/leaderboard/{}", self.config.server_url, period)
    }

    /// Build the URL for score submission
    #[allow(dead_code)]
    fn submit_url(&self) -> String {
        format!("{}/submit", self.config.server_url)
    }
}

#[cfg(test)]
mod client_tests {
    use super::*;

    fn enabled_config() -> OnlineConfig {
        OnlineConfig {
            enabled: true,
            server_url: "https://test.example.com/api".to_string(),
            player_name: Some("TestPlayer".to_string()),
            signature_secret: Some("test_secret".to_string()),
        }
    }

    fn disabled_config() -> OnlineConfig {
        OnlineConfig {
            enabled: false,
            ..Default::default()
        }
    }

    #[test]
    fn test_client_disabled() {
        let client = LeaderboardClient::new(disabled_config());
        assert!(!client.is_enabled());
        
        let result = client.fetch_leaderboard(LeaderboardPeriod::Daily, 10);
        assert!(matches!(result, Err(OnlineError::Disabled)));
    }

    #[test]
    fn test_client_enabled() {
        let client = LeaderboardClient::new(enabled_config());
        assert!(client.is_enabled());
        assert_eq!(client.player_name(), Some("TestPlayer"));
    }

    #[test]
    fn test_submit_score_no_player_name() {
        let config = OnlineConfig {
            enabled: true,
            player_name: None,
            ..Default::default()
        };
        let client = LeaderboardClient::new(config);
        let result = client.submit_score(1000, 5, 100, 12345);
        
        assert!(matches!(result, Err(OnlineError::NoPlayerName)));
    }

    #[test]
    fn test_url_building() {
        let client = LeaderboardClient::new(enabled_config());
        
        assert_eq!(
            client.leaderboard_url(LeaderboardPeriod::Daily),
            "https://test.example.com/api/leaderboard/daily"
        );
        assert_eq!(
            client.submit_url(),
            "https://test.example.com/api/submit"
        );
    }

    // Mock tests only run when online feature is disabled
    #[cfg(not(feature = "online"))]
    #[test]
    fn test_fetch_leaderboard_mock() {
        let client = LeaderboardClient::new(enabled_config());
        let result = client.fetch_leaderboard(LeaderboardPeriod::Weekly, 10);
        
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.period, LeaderboardPeriod::Weekly);
    }

    #[cfg(not(feature = "online"))]
    #[test]
    fn test_submit_score_mock() {
        let client = LeaderboardClient::new(enabled_config());
        let result = client.submit_score(1000, 5, 100, 12345);
        
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.success);
    }
}

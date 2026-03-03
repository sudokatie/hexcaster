//! Daily challenge system for Hexcaster.
//! 
//! Provides date-based seeded dungeons so everyone plays the same layout each day.

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use chrono::{Datelike, Local, NaiveDate};
use serde::{Deserialize, Serialize};

/// Get the seed for today's daily challenge.
pub fn today_seed() -> u64 {
    seed_for_date(Local::now().date_naive())
}

/// Get the seed for a specific date.
pub fn seed_for_date(date: NaiveDate) -> u64 {
    // Combine year, month, day into a deterministic seed
    // Using prime multipliers for better distribution
    let year = date.year() as u64;
    let month = date.month() as u64;
    let day = date.day() as u64;
    
    year.wrapping_mul(31337)
        .wrapping_add(month.wrapping_mul(1337))
        .wrapping_add(day.wrapping_mul(37))
}

/// Format a date as a readable string (YYYY-MM-DD).
pub fn date_string(date: NaiveDate) -> String {
    date.format("%Y-%m-%d").to_string()
}

/// Get today's date string.
pub fn today_string() -> String {
    date_string(Local::now().date_naive())
}

/// A score entry for the leaderboard.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreEntry {
    pub name: String,
    pub score: u32,
    pub floor_reached: i32,
    pub turns_taken: u32,
    pub timestamp: i64,
}

impl ScoreEntry {
    pub fn new(name: impl Into<String>, score: u32, floor_reached: i32, turns_taken: u32) -> Self {
        Self {
            name: name.into(),
            score,
            floor_reached,
            turns_taken,
            timestamp: chrono::Utc::now().timestamp(),
        }
    }
}

/// Daily leaderboard storage.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Leaderboard {
    /// Scores keyed by date string (YYYY-MM-DD).
    pub scores: BTreeMap<String, Vec<ScoreEntry>>,
}

impl Leaderboard {
    /// Load leaderboard from file.
    pub fn load() -> Self {
        let path = Self::path();
        if path.exists() {
            match fs::read_to_string(&path) {
                Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
                Err(_) => Self::default(),
            }
        } else {
            Self::default()
        }
    }

    /// Save leaderboard to file.
    pub fn save(&self) -> std::io::Result<()> {
        let path = Self::path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)
    }

    /// Add a score for today.
    pub fn add_score(&mut self, entry: ScoreEntry) {
        let date = today_string();
        let entries = self.scores.entry(date).or_default();
        entries.push(entry);
        // Sort by score descending
        entries.sort_by(|a, b| b.score.cmp(&a.score));
        // Keep only top 10
        entries.truncate(10);
    }

    /// Get today's scores.
    pub fn today_scores(&self) -> &[ScoreEntry] {
        let date = today_string();
        self.scores.get(&date).map(|v| v.as_slice()).unwrap_or(&[])
    }

    /// Get scores for a specific date.
    pub fn scores_for_date(&self, date: &str) -> &[ScoreEntry] {
        self.scores.get(date).map(|v| v.as_slice()).unwrap_or(&[])
    }

    /// Get the leaderboard file path.
    fn path() -> PathBuf {
        dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("hexcaster")
            .join("leaderboard.json")
    }
}

/// Calculate score based on run performance.
pub fn calculate_score(floor_reached: i32, turns_taken: u32, health_remaining: i32, enemies_killed: u32) -> u32 {
    let floor_bonus = (floor_reached as u32) * 1000;
    let efficiency_bonus = 10000_u32.saturating_sub(turns_taken * 10);
    let health_bonus = (health_remaining.max(0) as u32) * 10;
    let kill_bonus = enemies_killed * 50;
    
    floor_bonus + efficiency_bonus + health_bonus + kill_bonus
}

/// Generate a shareable code for a score.
pub fn share_code(date: &str, score: u32) -> String {
    // Simple shareable format: HEXCASTER-DATE-SCORE
    format!("HEXCASTER-{}-{}", date.replace('-', ""), score)
}

/// Parse a share code to get date and score.
pub fn parse_share_code(code: &str) -> Option<(String, u32)> {
    let parts: Vec<&str> = code.split('-').collect();
    if parts.len() == 3 && parts[0] == "HEXCASTER" {
        let date_str = parts[1];
        if date_str.len() == 8 {
            let date = format!(
                "{}-{}-{}",
                &date_str[0..4],
                &date_str[4..6],
                &date_str[6..8]
            );
            let score = parts[2].parse().ok()?;
            return Some((date, score));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_seed_for_date() {
        let date1 = NaiveDate::from_ymd_opt(2026, 3, 3).unwrap();
        let date2 = NaiveDate::from_ymd_opt(2026, 3, 4).unwrap();
        
        let seed1 = seed_for_date(date1);
        let seed2 = seed_for_date(date2);
        
        // Same date should give same seed
        assert_eq!(seed1, seed_for_date(date1));
        // Different dates should give different seeds
        assert_ne!(seed1, seed2);
    }

    #[test]
    fn test_seed_deterministic() {
        let date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        let seed1 = seed_for_date(date);
        let seed2 = seed_for_date(date);
        assert_eq!(seed1, seed2);
    }

    #[test]
    fn test_date_string() {
        let date = NaiveDate::from_ymd_opt(2026, 3, 15).unwrap();
        assert_eq!(date_string(date), "2026-03-15");
    }

    #[test]
    fn test_calculate_score() {
        // Floor 3, 100 turns, 50 health, 10 kills
        let score = calculate_score(3, 100, 50, 10);
        // 3000 (floor) + 9000 (efficiency) + 500 (health) + 500 (kills) = 13000
        assert_eq!(score, 13000);
    }

    #[test]
    fn test_share_code() {
        let code = share_code("2026-03-15", 12345);
        assert_eq!(code, "HEXCASTER-20260315-12345");
    }

    #[test]
    fn test_parse_share_code() {
        let code = "HEXCASTER-20260315-12345";
        let (date, score) = parse_share_code(code).unwrap();
        assert_eq!(date, "2026-03-15");
        assert_eq!(score, 12345);
    }

    #[test]
    fn test_parse_share_code_invalid() {
        assert!(parse_share_code("INVALID").is_none());
        assert!(parse_share_code("HEXCASTER-123-456").is_none());
    }

    #[test]
    fn test_score_entry() {
        let entry = ScoreEntry::new("Player1", 5000, 2, 150);
        assert_eq!(entry.name, "Player1");
        assert_eq!(entry.score, 5000);
        assert_eq!(entry.floor_reached, 2);
        assert_eq!(entry.turns_taken, 150);
    }

    #[test]
    fn test_leaderboard_add_score() {
        let mut lb = Leaderboard::default();
        lb.add_score(ScoreEntry::new("A", 1000, 1, 100));
        lb.add_score(ScoreEntry::new("B", 2000, 2, 90));
        lb.add_score(ScoreEntry::new("C", 500, 1, 120));
        
        let scores = lb.today_scores();
        assert_eq!(scores.len(), 3);
        // Should be sorted by score descending
        assert_eq!(scores[0].name, "B");
        assert_eq!(scores[1].name, "A");
        assert_eq!(scores[2].name, "C");
    }
}
